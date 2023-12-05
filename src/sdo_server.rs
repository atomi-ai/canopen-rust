use embedded_can::{Frame, StandardId};
use embedded_can::nb::Can;

use crate::cmd_header::{
    SdoBlockDownloadInitiateCmd, SdoBlockUploadCmd, SdoDownloadInitiateCmd, SdoDownloadSegmentCmd,
    SdoEndBlockDownloadCmd, SdoInitBlockUploadCmd,
};
use crate::error::CanAbortCode;
use crate::info;
use crate::node::Node;
use crate::prelude::*;
use crate::sdo_server::SdoState::{
    ConfirmUploadSdoBlock, DownloadSdoBlock, EndSdoBlockDownload, Normal, SdoSegmentDownload,
    SdoSegmentUpload, StartSdoBlockUpload,
};
use crate::util::{crc16_canopen_with_lut, flatten, genf_and_padding};

/// Represents the various states of the SDO (Service Data Object) communication process.
/// These states govern the different phases or modes of SDO transmissions in a CANopen system.
pub enum SdoState {
    /// The default state of SDO communication where no specific SDO operation is active.
    Normal,

    /// The state when segments of data are being uploaded from the server to the client.
    SdoSegmentUpload,

    /// The state when segments of data are being downloaded from the client to the server.
    SdoSegmentDownload,

    /// The state when blocks of data are being downloaded in a block-wise manner from the client to the server.
    DownloadSdoBlock,

    /// The state marking the conclusion of a block download process.
    EndSdoBlockDownload,

    /// The state marking the beginning of a block upload process.
    StartSdoBlockUpload,

    /// The state where the server waits for the client's confirmation after uploading blocks of data.
    ConfirmUploadSdoBlock,
}

impl<CAN: Can> Node<CAN> where CAN::Frame: Frame + Debug {
    fn create_error_frame(&self, error_code: CanAbortCode, index: u16, sub_index: u8) -> CAN::Frame {
        let mut packet = Vec::new();
        packet.push(0x80);
        packet.push((index & 0xFF) as u8);
        packet.push((index >> 8) as u8);
        packet.push(sub_index);
        let code_bytes = error_code.code().to_le_bytes();
        packet.extend_from_slice(&code_bytes);
        Frame::new(
            StandardId::new(0x580 | self.node_id).unwrap(),
            packet.as_slice(),
        )
        .unwrap()
    }

    fn genf(&self, data: &[u8]) -> Result<CAN::Frame, CanAbortCode> {
        Ok(genf_and_padding(0x580 | self.node_id, data))
    }

    pub(crate) fn gen_frame(
        &self,
        cmd: u8,
        index: u16,
        sub_index: u8,
        data: &Vec<u8>,
    ) -> Result<CAN::Frame, CanAbortCode> {
        let bytes = flatten(&[&[cmd], &index.to_le_bytes(), &sub_index.to_le_bytes(), data]);
        self.genf(bytes.as_slice())
    }

    pub(crate) fn next_state(
        &mut self,
        state: SdoState,
        res: Result<CAN::Frame, CanAbortCode>,
    ) -> Result<CAN::Frame, CanAbortCode> {
        self.sdo_state = state;
        res
    }

    pub(crate) fn dispatch_sdo_request(&mut self, frame: &CAN::Frame) -> Option<CAN::Frame> {
        if self.filter_frame(frame) {
            return None;
        }
        let cmd = frame.data()[0];
        let ccs = cmd >> 5;

        let index = u16::from_le_bytes([frame.data()[1], frame.data()[2]]);
        let sub_index = frame.data()[3];
        let res = match &self.sdo_state {
            SdoSegmentDownload => {
                let res = self.download_segment(frame.data());
                self.next_state(Normal, res)
            }
            SdoSegmentUpload => self.upload_segment(cmd),
            DownloadSdoBlock => self.block_download(frame.data()),
            EndSdoBlockDownload => {
                let res = self.end_block_download(frame.data());
                self.next_state(Normal, res)
            }
            StartSdoBlockUpload => self.start_block_upload(frame.data()),
            ConfirmUploadSdoBlock => self.confirm_block_upload(frame.data()),
            Normal => {
                // ccs: 0x1 / 0x2 / 0x6 / 0x5
                match ccs {
                    0x1 => self.initiate_download(index, sub_index, frame.data()),
                    0x2 => self.initiate_upload(index, sub_index),
                    0x6 => self.init_block_download(index, sub_index, frame.data()),
                    0x5 => self.init_block_upload(index, sub_index, frame.data()),
                    _ => Err(CanAbortCode::CommandSpecifierNotValidOrUnknown),
                }
            }
        };

        match res {
            Ok(frame) => Some(frame),
            Err(code) => {
                let (idx, sidx) = match self.sdo_state {
                    Normal => (index, sub_index),
                    _ => (self.reserved_index, self.reserved_sub_index),
                };
                self.sdo_state = Normal;
                self.read_buf = None;
                self.write_buf = None;
                self.need_crc = false;
                Some(self.create_error_frame(code, idx, sidx))
            }
        }
    }

    fn initiate_upload(&mut self, index: u16, sub_index: u8) -> Result<CAN::Frame, CanAbortCode> {
        let data = match self.object_directory.get_variable(index, sub_index) {
            Ok(var) => var.default_value().data(),
            Err(code) => return Err(code),
        };

        if data.len() <= 4 {
            if data.len() == 0 {
                // empty string. Need to have a solution.
                todo!("zephyr")
            }
            let cmd = 0x43 | ((((4 - data.len()) as u8) & 0x3) << 2);
            let t = data.clone();
            return self.gen_frame(cmd, index, sub_index, &t);
        }

        self.read_buf = Some(data.clone());
        self.read_buf_index = 0;
        self.next_read_toggle = 0;
        self.reserved_index = index;
        self.reserved_sub_index = sub_index;
        let len_bytes_vec = (data.len() as u32).to_le_bytes().to_vec();
        let res = self.gen_frame(0x41, index, sub_index, &len_bytes_vec);
        self.next_state(SdoSegmentUpload, res)
    }

    fn upload_segment(&mut self, cmd: u8) -> Result<CAN::Frame, CanAbortCode> {
        if cmd >> 5 != 0x3 {
            return Err(CanAbortCode::GeneralError);
        }
        // Ensure buffer is not None and has data left to send
        if let Some(buffer) = &self.read_buf {
            let toggle = (cmd >> 4) & 0x1;
            if toggle != self.next_read_toggle {
                return Err(CanAbortCode::ToggleBitNotAlternated);
            }
            self.next_read_toggle ^= 1;
            let remaining_data = &buffer[self.read_buf_index..];

            if remaining_data.len() > 7 {
                // Extract the next 7 bytes from the buffer
                let data = flatten(&[&[toggle << 4], &remaining_data[..7]]);
                self.read_buf_index += 7;
                self.genf(data.as_slice())
            } else {
                // For the remaining data, set n to the length of the data and c=1
                let n = 7 - remaining_data.len() as u8;
                let data = flatten(&[&[0x01 | (toggle << 4) | (n << 1)], remaining_data]);
                self.read_buf = None;
                self.read_buf_index = 0;

                let res = self.genf(data.as_slice());
                self.next_state(Normal, res)
            }
        } else {
            Err(CanAbortCode::GeneralError)
        }
    }

    fn initiate_download(
        &mut self,
        index: u16,
        sub_index: u8,
        req: &[u8],
    ) -> Result<CAN::Frame, CanAbortCode> {
        let cmd = SdoDownloadInitiateCmd::from(req[0]);
        if cmd.e() && cmd.s() {
            // Expedite download.
            let data = &req[4..(8 - cmd.n() as usize)];
            return match self.set_value_with_check(index, sub_index, data) {
                Err(code) => Err(code),
                Ok(_) => self.gen_frame(0x60, index, sub_index, &vec![0, 0, 0, 0]),
            };
        }

        // normal download
        self.write_buf = Some(Vec::new());
        self.reserved_index = index;
        self.reserved_sub_index = sub_index;

        if cmd.s() {
            match req[4..].try_into() {
                Ok(arr) => self.write_data_size = u32::from_le_bytes(arr),
                Err(_) => return Err(CanAbortCode::GeneralError),
            }
        } else {
            self.write_data_size = 0;
        }
        let res = self.gen_frame(0x60, index, sub_index, &vec![0, 0, 0, 0]);
        self.next_state(SdoSegmentDownload, res)
    }

    fn download_segment(&mut self, req: &[u8]) -> Result<CAN::Frame, CanAbortCode> {
        let req_cmd = SdoDownloadSegmentCmd::from(req[0]);
        if req_cmd.ccs() != 0x0 {
            return Err(CanAbortCode::GeneralError);
        }

        if let Some(buf) = &mut self.write_buf {
            let resp_cmd = 0x20 | (req_cmd.t() << 4);
            if !req_cmd.c() {
                // Not finish, push data and continue.
                buf.extend_from_slice(&req[1..]);
                return self.genf(&[resp_cmd]);
            }

            // No more segments to be downloaded
            buf.extend_from_slice(&req[1..(8 - req_cmd.n() as usize)]);
            if self.write_data_size > 0 && self.write_data_size as usize != buf.len() {
                // 事先大小跟最终大小不符，报错。
                return Err(CanAbortCode::GeneralError);
            }
            let (i, si) = (self.reserved_index, self.reserved_sub_index);
            let t = buf.clone();
            match self.set_value_with_check(i, si, t.as_slice()) {
                Ok(_) => self.genf(&[resp_cmd]),
                Err(code) => Err(code),
            }
        } else {
            Err(CanAbortCode::GeneralError)
        }
    }

    fn init_block_download(
        &mut self,
        index: u16,
        sub_index: u8,
        req: &[u8],
    ) -> Result<CAN::Frame, CanAbortCode> {
        let cmd = SdoBlockDownloadInitiateCmd::from(req[0]);
        if cmd.cc() {
            self.need_crc = true;
        }
        if cmd.s() {
            match req[4..8].try_into() {
                Ok(arr) => self.write_data_size = u32::from_le_bytes(arr),
                Err(_) => return Err(CanAbortCode::GeneralError),
            }
        } else {
            self.write_data_size = 0;
        }

        // Prepare to download.
        self.write_buf = Some(Vec::new());
        self.current_seq_number = 0;
        self.reserved_index = index;
        self.reserved_sub_index = sub_index;

        let resp_cmd = 0xA0 | ((self.crc_enabled as u8) << 2);
        let v = &vec![self.block_size, 0, 0, 0];
        let res = self.gen_frame(resp_cmd, index, sub_index, v);
        self.next_state(DownloadSdoBlock, res)
    }

    fn set_value_with_check(&mut self, index: u16, sub_index: u8, data: &[u8]) -> Result<(), CanAbortCode> {
        // pdo precheck
        match index {
            0x1600..=0x17FF | 0x1A00..=0x1BFF => {
                if sub_index > 0 && sub_index <= crate::pdo::MAX_PDO_MAPPING_LENGTH {
                    if data.len() != 4 {
                        info!("set_value_with_check() 1.1, index = {:#?}, sub_index = {}, data = {:?}", index, sub_index, data);
                        return Err(CanAbortCode::ObjectCannotBeMappedToPDO);
                    }
                    let di = (data[3] as u16) << 8 | (data[2] as u16);
                    let d_si = data[1];
                    match self.object_directory.get_variable(di, d_si) {
                        Ok(var) => {
                            if !var.pdo_mappable() {
                                return Err(CanAbortCode::ObjectCannotBeMappedToPDO)
                            }
                            if index < 0x1800 && !var.access_type().is_writable() {
                                return Err(CanAbortCode::ObjectCannotBeMappedToPDO)
                            }
                        }
                        Err(err) => { return Err(err); }
                    }
                }
            }
            _ => {}
        }

        match self.object_directory.set_value(index, sub_index, data, false) {
            Ok(var) => {
                // Post check
                match index {
                    0x1400..=0x1BFF => {
                        // Update PDO related parameters
                        let var_clone = var.clone();
                        self.update(&var_clone)
                    },
                    0x1017 => {
                        let t: u16 = var.default_value().to();
                        self.heartbeats_timer = t as u32;
                        Ok(())
                    }
                    _ => {Ok(())}
                }
            }
            Err(code) => {Err(code)}
        }
    }

    fn block_download(&mut self, req: &[u8]) -> Result<CAN::Frame, CanAbortCode> {
        let seqno = req[0] & 0x7F;
        self.current_seq_number += 1;
        if seqno != self.current_seq_number {
            return Err(CanAbortCode::GeneralError);
        }

        if let Some(buf) = &mut self.write_buf {
            buf.extend_from_slice(&req[1..]);
            if req[0] >> 7 == 1 {
                // no more segments
                // Resize to preset data size.
                if buf.len() >= self.write_data_size as usize
                    && buf.len() - 7 < self.write_data_size as usize
                {
                    buf.resize(self.write_data_size as usize, 0);
                }
                // TODO(zephyr): check correctness: CRC
                // Write data to object directory.
                let (i, si) = (self.reserved_index, self.reserved_sub_index);
                // TODO(zephyr): Don't clone in set value in the future.
                let t = buf.clone();
                match self.set_value_with_check(i, si, t.as_slice()) {
                    Ok(_) => {}
                    Err(code) => return Err(code),
                }
                let (c, b) = (self.current_seq_number, self.block_size);
                let res = self.genf(&[0xA2, c, b]);
                self.next_state(EndSdoBlockDownload, res)
            } else {
                self.genf(&[])
            }
        } else {
            Err(CanAbortCode::GeneralError)
        }
    }

    fn end_block_download(&self, req: &[u8]) -> Result<CAN::Frame, CanAbortCode> {
        let cmd = SdoEndBlockDownloadCmd::from(req[0]);
        if cmd.n() as u32 != 7 - self.write_data_size % 7 {
            return Err(CanAbortCode::GeneralError);
        }
        // TODO(zephyr): CRC check.
        let _crc = u16::from_le_bytes([req[1], req[2]]);

        self.genf(&[0xA1])
    }

    fn init_block_upload(
        &mut self,
        index: u16,
        sub_index: u8,
        req: &[u8],
    ) -> Result<CAN::Frame, CanAbortCode> {
        let cmd = SdoInitBlockUploadCmd::from(req[0]);
        let (blk_size, _pst) = (req[4], req[5]);

        if cmd.ccs() != 0x5 || cmd.cs() != 0 {
            return Err(CanAbortCode::GeneralError);
        }
        if blk_size >= 0x80 {
            return Err(CanAbortCode::InvalidBlockSize);
        }

        // Init setting for upload (read)
        self.need_crc = cmd.cc();
        self.block_size = blk_size;
        self.reserved_index = index;
        self.reserved_sub_index = sub_index;
        match self.object_directory.get_variable(index, sub_index) {
            Ok(var) => {
                self.read_buf = Some(var.default_value().data().clone());
                self.read_buf_index = 0;
            }
            Err(code) => return Err(code),
        }

        // Prepare the response packet.
        let resp_cmd = 0xC2 | (self.crc_enabled as u8) << 2;
        let v: [u8; 4] = (self.read_buf.as_ref().unwrap().len() as u32).to_le_bytes();
        let res = self.gen_frame(resp_cmd, index, sub_index, &v.to_vec());
        self.next_state(StartSdoBlockUpload, res)
    }

    fn start_block_upload(&mut self, req: &[u8]) -> Result<CAN::Frame, CanAbortCode> {
        let cmd = SdoBlockUploadCmd::from(req[0]);
        if cmd.ccs() != 0x5 || cmd.cs() != 0x3 {
            return Err(CanAbortCode::GeneralError);
        }

        // Start to send frames (server => client)
        let buf = self.read_buf.as_ref().unwrap();
        // TODO(zephyr): 需要另外考虑的几个情况：
        // - total_seqs > blksize：需要好几个block，每个block若干seqs来进行传输。
        // - 根据ackseq进行重传的逻辑。（这个download也需要考虑）
        let total_seqs = ((buf.len() - 1) / 7 + 1) as u8;
        for i in 0..total_seqs - 1 {
            // This is a special case, directly transmit (total_seq - 1) frames,
            // only leave the last one at last for change the state.
            let (s, e) = ((i * 7) as usize, (i * 7 + 7) as usize);
            let frame = genf_and_padding(
                0x580 | self.node_id,
                flatten(&[&[i + 1], &buf[s..e]]).as_slice(),
            );
            self.can_network
                .transmit(&frame)
                .expect("error in uploading blocks");
        }
        let s = ((total_seqs - 1) * 7) as usize;
        let f = self.genf(flatten(&[&[total_seqs | 0x80], &buf[s..]]).as_slice());
        self.next_state(ConfirmUploadSdoBlock, f)
    }

    fn confirm_block_upload(&mut self, req: &[u8]) -> Result<CAN::Frame, CanAbortCode> {
        let cmd = SdoBlockUploadCmd::from(req[0]);
        if cmd.ccs() != 0x5 || cmd.cs() != 2 {
            return Err(CanAbortCode::GeneralError);
        }
        let buf = self.read_buf.as_ref().unwrap();
        let (ackseq, blksize) = (req[1], req[2]);
        if ackseq as usize != (buf.len() - 1) / 7 + 1 {
            return Err(CanAbortCode::CommandSpecifierNotValidOrUnknown);
        }

        // Don't understand whether it is the right place to modify this.
        self.block_size = blksize;

        let n = (7 - buf.len() % 7) as u8;
        let resp_cmd = 0xC1 | (n << 2);
        let crc: u16 = if self.need_crc {
            crc16_canopen_with_lut(buf.as_slice())
        } else {
            0
        };
        let t = flatten(&[&[resp_cmd], &crc.to_le_bytes(), &[0, 0, 0, 0, 0]]);
        self.genf(t.as_slice())
    }
}

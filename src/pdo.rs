use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;

use embedded_can::Frame;
use embedded_can::nb::Can;
use hashbrown::HashMap;
use log::trace;

use crate::error::{AbortCode, ErrorCode};
use crate::{debug, info};
use crate::error::AbortCode::ExceedPDOSize;
use crate::node::{Node, NodeEvent};
use crate::object_directory::Variable;
use crate::util::{create_frame, make_abort_error, vec_to_u64};

pub(crate) const MAX_PDO_MAPPING_LENGTH: u8 = 64;

#[derive(Debug, Clone)]
pub struct PdoObject {
    // Properties
    is_pdo_valid: bool,
    _not_used_rtr_allowed: bool,
    _not_used_is_29bit_can_id: bool,  // to differentiate CAN2.0A / CAN2.0B

    // Communication relativeX
    largest_sub_index: u8,
    cob_id: u16,
    transmission_type: u8,
    inhibit_time: u16,
    event_timer: u16,

    // Mapping relative
    num_of_map_objs: u8,
    mappings: [(u16, u8, u8); MAX_PDO_MAPPING_LENGTH as usize],
    // index, sub_index, length
    total_length: u8,

    // Used by RPDO only. Because for TPDO, we may need to transfer data out in high frequency,
    // it isn't suitable to cache high freq data here.
    cached_data: Vec<u8>,
}

impl PdoObject {
    pub fn total_length(&self) -> u8 {
        self.total_length
    }
    pub fn largest_sub_index(&self) -> u8 {
        self.largest_sub_index
    }
    pub fn cob_id(&self) -> u16 {
        self.cob_id
    }
    pub fn transmission_type(&self) -> u8 {
        self.transmission_type
    }
    pub fn event_timer(&self) -> u16 {
        self.event_timer
    }

    pub fn set_cached_data(&mut self, cached_data: &[u8]) {
        self.cached_data.clear();
        self.cached_data.extend_from_slice(cached_data);
    }
    pub fn clear_cached_data(&mut self) {
        self.cached_data.clear();
    }
}

impl PdoObject {
    fn update_comm_params(&mut self, var: &Variable) -> Option<u16> {
        match var.sub_index() {
            0 => self.largest_sub_index = var.default_value().to(),
            1 => {
                let t: u32 = var.default_value().to();
                self.is_pdo_valid = (t >> 31 & 0x1) == 0;
                self._not_used_rtr_allowed = (t >> 30 & 0x1) == 1;
                self._not_used_is_29bit_can_id = (t >> 29 & 0x1) == 1;
                self.cob_id = (t & 0xFFFF) as u16;
            }
            2 => self.transmission_type = var.default_value().to(),
            3 => self.inhibit_time = var.default_value().to(),
            5 => self.event_timer = var.default_value().to(),
            _ => {}
        }
        Some(self.cob_id)
    }

    fn update_map_params(&mut self, var: &Variable) -> Option<u16> {
        // info!("xfguo: update_map_params() 0. var = {:#x?}", var);
        if var.sub_index() == 0 {
            let t = var.default_value().to();
            self.num_of_map_objs = t;
            // if var.index == 0x1A01 {
            //     info!("xfguo: update_map_params() 1.1, var = {:#x?}, t = {}", var, t);
            // }
        } else {
            let t: u32 = var.default_value().to();
            let si = var.sub_index() as usize;
            self.mappings[si - 1] =
                ((t >> 16) as u16, ((t >> 8) & 0xFF) as u8, (t & 0xFF) as u8);
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct PdoObjects {
    pdos: [Option<PdoObject>; 8],
    cob_to_index: HashMap<u16, usize>,
}

impl PdoObjects {
    pub fn new() -> Self {
        let default_pdo = PdoObject {
            is_pdo_valid: false,
            _not_used_rtr_allowed: false,
            _not_used_is_29bit_can_id: false,
            largest_sub_index: 5,
            cob_id: 0,
            transmission_type: 0x01,
            inhibit_time: 0,
            event_timer: 0,
            num_of_map_objs: 0,
            mappings: [(0, 0, 0); MAX_PDO_MAPPING_LENGTH as usize],
            total_length: 0,
            cached_data: vec![],
        };
        let pdos = [(); 8].map(|_| Some(default_pdo.clone()));
        PdoObjects { pdos, cob_to_index: HashMap::new() }
    }

    pub fn get_mut_rpdo_with_cob_id(&mut self, cob_id: u16) -> Result<&mut PdoObject, ErrorCode> {
        let index = *self.cob_to_index.get(&cob_id).ok_or(ErrorCode::NoCobIdInRpdo {cob_id})?;
        let pdo = self.pdos[index].as_mut().ok_or(ErrorCode::NoPdoObjectInIndex {index})?;
        Ok(pdo)
    }
}

fn should_trigger_pdo(is_sync: bool, event: NodeEvent, transmission_type: u32, event_times: u32, count: u32) -> bool {
    if is_sync {
        if transmission_type == 0 || transmission_type > 240 || count % transmission_type != 0 {
            // info!("xfguo: should_trigger_pdo 1.1.1, count = {}, transmission_type = {}", count, transmission_type);
            return false;
        }
    } else {
        if event == NodeEvent::NodeStart { return true; }
        if transmission_type != 0xFE && transmission_type != 0xFF {
            // info!("xfguo: transmit_pdo_messages 1.1.2, count = {}, tt = {}", count, transmission_type);
            return false;
        }
        if event_times == 0 || count % event_times != 0 {
            // info!("xfguo: transmit_pdo_messages 1.1.3, count = {}, event_timer = {}", count, event_times);
            return false;
        }
    }
    true
}

impl<CAN: Can> Node<CAN> where CAN::Frame: Frame + Debug {

    // RPDO section
    pub(crate) fn save_rpdo_messages(&mut self, is_sync: bool, event: NodeEvent, count: u32) {
        for pdo in self.pdo_objects.pdos[0..4].iter_mut().filter_map(|x| x.as_mut()) {
            let tt = pdo.transmission_type as u32;

            if !pdo.is_pdo_valid
                || !should_trigger_pdo(is_sync, event, tt, pdo.event_timer as u32, count)
                || pdo.cached_data.is_empty() {
                continue
            }

            debug!("save_rpdo_messages() 1.3, count = {}, pdo = {:#x?} ", count, pdo);
            let mapping_lengths: Vec<u8> = pdo.mappings[..pdo.num_of_map_objs as usize]
                .iter()
                .map(|(_, _, l)| *l)
                .collect();

            let unpacked_data = unpack_data(&pdo.cached_data, &mapping_lengths);
            if unpacked_data.len() < pdo.num_of_map_objs as usize {
                // TODO(zephyr): Error, do we need to send EMGY msg?
                info!("error: unmatch length: unpacked_data = {:?}, mapping = {:?}", unpacked_data, pdo.mappings);
                continue;
            }

            for (idx, &(i, si, _)) in pdo.mappings.iter().enumerate().take(pdo.num_of_map_objs as usize) {
                let (data, _) = unpacked_data[idx];
                self.object_directory.set_value_with_fitting_size(i, si, &data.to_le_bytes());
            }

            pdo.clear_cached_data();
        }
    }

    fn validate_pdo_mappings(&mut self, pdo: &PdoObject, index: u16) -> Result<(), ErrorCode> {
        for si in (1..=pdo.num_of_map_objs as usize).rev() {
            self.object_directory.get_variable(index, si as u8)
                .map_err(|_| make_abort_error(AbortCode::ObjectCannotBeMappedToPDO, "".to_string()))?;
        }
        Ok(())
    }

    fn calculate_total_length(pdo: &PdoObject) -> u8 {
        pdo.mappings.iter()
            .take(pdo.num_of_map_objs as usize)
            .map(|mapping| mapping.2)
            .sum()
    }

    pub(crate) fn update(&mut self, var: &Variable) -> Result<(), ErrorCode> {
        let (pdo_type, pdo_index) = (var.index() >> 8, (var.index() & 0xF) as usize);
        if !(0x14..0x1C).contains(&pdo_type) {
            return Ok(());
        }
        let index = pdo_index + (pdo_type >= 0x18) as usize * 4;
        let mut pdo = self.pdo_objects.pdos[index].take().ok_or(
            ErrorCode::NoPdoObjectInIndex {index})?;
        let result = (|| -> Result<(), ErrorCode> {
            if pdo_type & 0x3 < 2 {
                pdo.update_comm_params(var);
                self.pdo_objects.cob_to_index.insert(pdo.cob_id, pdo_index);
            } else {
                pdo.update_map_params(var);
                if var.sub_index() == 0 {
                    self.validate_pdo_mappings(&pdo, var.index())?;
                    pdo.total_length = Node::<CAN>::calculate_total_length(&pdo);
                    if pdo.total_length > MAX_PDO_MAPPING_LENGTH {
                        return Err(make_abort_error(ExceedPDOSize, "".to_string()));
                    }
                }
            }
            Ok(())
        })();
        self.pdo_objects.pdos[index] = Some(pdo);
        result
    }

    // TPDO section
    pub(crate) fn transmit_pdo_messages(&mut self, is_sync: bool, event: NodeEvent, count: u32)
        -> Result<(), ErrorCode> {
        trace!("xfguo: transmit_pdo_messages 0");
        for index in 4..8 {
            let pdo = self.pdo_objects.pdos[index].take().ok_or(ErrorCode::NoPdoObjectInIndex {index})?;
            let result = (|| -> Result<(), ErrorCode> {
                let tt = pdo.transmission_type as u32;
                if !pdo.is_pdo_valid || !should_trigger_pdo(is_sync, event, tt, pdo.event_timer as u32, count) {
                    return Ok(())
                }

                debug!("xfguo: transmit_pdo_messages 2, count = {}, pdo[{}] = {:x?}", count, index, pdo);
                // Emit a TPDO message.
                let mappings = pdo.mappings[..pdo.num_of_map_objs as usize].to_vec();
                let frame = self.gen_pdo_frame(pdo.cob_id, pdo.num_of_map_objs, mappings)?;
                self.transmit(&frame);
                Ok(())
            })();
            self.pdo_objects.pdos[index] = Some(pdo);
            result?
        }
        Ok(())
    }

    fn gen_pdo_frame(&mut self, cob_id: u16, num_of_map_objs: u8, mappings: Vec<(u16, u8, u8)>)
                                -> Result<CAN::Frame, ErrorCode> {
        let mut data_pairs = Vec::new();
        for (idx, sub_idx, bits) in mappings.iter().take(num_of_map_objs as usize) {
            let variable = self.object_directory.get_variable(*idx, *sub_idx)
                .map_err(|_| ErrorCode::VariableNotFound { index: *idx, sub_index: *sub_idx })?;

            let data = vec_to_u64(variable.default_value().data());
            data_pairs.push((data, *bits));
        }

        let packet = pack_data(&data_pairs);
        create_frame(cob_id, &packet)
    }
}

fn pack_data(vec: &[(u64, u8)]) -> Vec<u8> {
    let mut merged = 0u64;
    let mut total_bits = 0usize;

    for &(data, bits) in vec.iter().rev() {
        merged |= (data & ((1 << bits) - 1)) << total_bits;
        total_bits += bits as usize;
    }

    merged.to_be_bytes()[8 - (total_bits + 7) / 8..].to_vec()
}

fn unpack_data(vec: &[u8], bits: &[u8]) -> Vec<(u64, u8)> {
    let mut data = vec_to_u64(vec);
    let mut res = Vec::new();

    for &bit in bits.iter().rev() {
        let mask = (1 << bit) - 1;
        res.push(((data & mask), bit));
        data >>= bit;
    }

    res.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cut_data_with_bits(vec: &Vec<(u64, u8)>) -> Vec<(u64, u8)> {
        let mut res: Vec<(u64, u8)> = Vec::new();
        for (data, bits) in vec {
            res.push((data & ((1 << bits) - 1), *bits));
        }
        res
    }

    #[test]
    fn test_data_to_packet_and_packet_to_data() {
        // Convert initial_data to a packet
        let initial_data = vec![
            (0xABCDu64, 12),
            (0x123456u64, 20),
            (0x0102u64, 9),
        ];
        let packet = pack_data(&initial_data);
        info!("xfguo: packet = {:?}", packet);

        let result_data = unpack_data(&packet, &vec![12, 20, 9]);
        info!("xfguo: unpacked = {:?}", result_data);

        let cutted_data = cut_data_with_bits(&initial_data);
        assert_eq!(result_data, cutted_data);
    }
}

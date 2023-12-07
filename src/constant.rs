use core::ops::Range;

/// Canopen Function code prefixes on COB_ID
pub(crate) const COB_FUNC_NMT: u16 = 0x000;
pub(crate) const COB_FUNC_SYNC: u16 = 0x080;
pub(crate) const COB_FUNC_RPDO_0: u16 = 0x200;
// pub(crate) const COB_FUNC_RPDO_1: u16 = 0x300;
// pub(crate) const COB_FUNC_RPDO_2: u16 = 0x400;
pub(crate) const COB_FUNC_RPDO_3: u16 = 0x500;
pub(crate) const COB_FUNC_TRANSMIT_SDO: u16 = 0x580;
pub(crate) const COB_FUNC_RECEIVE_SDO: u16 = 0x600;
pub(crate) const COB_FUNC_MASK: u16 = 0xFF80;


/// CANOPEN Registers
pub(crate) const REG_ERROR: u16 = 0x1001;
// pub(crate) const REG_MANUFACTURER_STATUE: u16 = 0x1002;
pub(crate) const REG_PRE_DEFINED_ERROR: u16 = 0x1003;
pub(crate) const REG_RESTORE_DEFAULT_PARAMETERS: u16 = 0x1011;
pub(crate) const REG_PRODUCER_HEARTBEAT_TIME: u16 = 0x1017;

pub(crate) const COMMUNICATION_REGISTERS_RANGE: Range<u16> = 0x1000..0x1FFF;
pub(crate) const APPLICATION_REGISTERS_RANGE: Range<u16> = 0x6000..0x9FFF;
pub(crate) const ALL_REGISTERS_RANGE: Range<u16> = 0x6000..0x9FFF;

/// Emergency Codes
pub(crate) const EMCY_PDO_NOT_PROCESSED: u16 = 0x8210;

/// Misc
pub(crate) const RESET_MAGIC_CODE: u32 = 0x64_61_6F_6C;

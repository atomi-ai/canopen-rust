use bitfield_struct::bitfield;

/// Represents the SDO (Service Data Object) Download Initiate Command.
/// The bitfield representation is based on an `u8` (8-bit unsigned integer), with the most significant bit (MSB) ordering.
#[bitfield(u8, order = Msb)]
pub struct SdoDownloadInitiateCmd {
    /// Command Specifier.
    /// Indicates the specific command to be executed within the SDO protocol.
    #[bits(3)]
    ccs: u8,

    /// Reserved bit.
    /// This bit is reserved for future use and should typically be set to 0.
    #[bits(1)]
    _reserved_0: u8,

    /// Number of data bytes that do not contain data, ranging from 0 to 3.
    /// For example, if `n` is 2, then the last 2 bytes of the 4-byte data field do not contain meaningful data.
    #[bits(2)]
    pub n: u8,

    /// Expedited Transfer flag.
    /// Indicates whether the SDO transfer is expedited (`true`) or segmented (`false`).
    #[bits(1)]
    pub e: bool,

    /// Size indicator.
    /// If set (`true`), indicates that the `n` field is valid and specifies the number of bytes without data.
    /// If unset (`false`), the `n` field should be ignored.
    #[bits(1)]
    pub s: bool,
}
/// Represents the SDO (Service Data Object) Download Segment Command.
/// The bitfield representation is based on an `u8` (8-bit unsigned integer) with the most significant bit (MSB) ordering.
#[bitfield(u8, order = Msb)]
pub struct SdoDownloadSegmentCmd {
    /// Command Specifier.
    /// Indicates the specific command to be executed within the SDO protocol for segmented transfers.
    #[bits(3)]
    pub ccs: u8,

    /// Toggle bit.
    /// Alternates for each subsequent segment during a segmented SDO transfer. Helps in ensuring the data integrity.
    #[bits(1)]
    pub t: u8,

    /// Number of data bytes that do not contain data, ranging from 0 to 7 in case of segment commands.
    /// For example, if `n` is 2, then the last 2 bytes of the 7-byte data field do not contain meaningful data.
    #[bits(3)]
    pub n: u8,

    /// More segments to follow indicator.
    /// If set (`true`), indicates that more segments will follow. If unset (`false`), it's the last segment of the SDO transfer.
    #[bits(1)]
    pub c: bool,
}

/// Represents the SDO (Service Data Object) Block Download Initiate Command.
/// The bitfield representation is based on an `u8` (8-bit unsigned integer) with the most significant bit (MSB) ordering.
#[bitfield(u8, order = Msb)]
pub struct SdoBlockDownloadInitiateCmd {
    /// Command Specifier.
    /// Indicates the specific command to be executed within the SDO protocol for block transfers.
    #[bits(3)]
    pub ccs: u8,

    /// Reserved bits.
    /// These bits are reserved for future use and should typically be set to 0.
    #[bits(2)]
    _reserved_0: u8,

    /// CRC support flag.
    /// If set (`true`), indicates that the SDO block download will use CRC for ensuring data integrity.
    #[bits(1)]
    pub cc: bool,

    /// Size indicator.
    /// If set (`true`), indicates that the size of the block is specified. If unset (`false`), the size of the block should be ignored.
    #[bits(1)]
    pub s: bool,

    /// Client subcommand.
    /// Specific subcommand for block download initiate in the SDO protocol.
    #[bits(1)]
    cs: bool,
}
/// Represents the SDO (Service Data Object) End Block Download Command.
/// The bitfield representation is based on an `u8` (8-bit unsigned integer) with the most significant bit (MSB) ordering.
#[bitfield(u8, order = Msb)]
pub struct SdoEndBlockDownloadCmd {
    /// Command Specifier.
    /// Indicates the specific command to be executed within the SDO protocol for ending block downloads.
    #[bits(3)]
    pub ccs: u8,

    /// Number of unused bytes in the last segment of the block download, ranging from 0 to 7.
    /// For example, if `n` is 2, then the last 2 bytes of the data field do not contain meaningful data.
    #[bits(3)]
    pub n: u8,

    /// Reserved bit.
    /// This bit is reserved for future use and should typically be set to 0 or left unset.
    #[bits(1)]
    _reserved_0: bool,

    /// Client subcommand.
    /// Specific subcommand for ending block download in the SDO protocol.
    #[bits(1)]
    pub cs: bool,
}

/// Represents the SDO (Service Data Object) Initiate Block Upload Command.
/// The bitfield representation is based on an `u8` (8-bit unsigned integer) with the most significant bit (MSB) ordering.
#[bitfield(u8, order = Msb)]
pub struct SdoInitBlockUploadCmd {
    /// Command Specifier.
    /// Indicates the specific command to be executed within the SDO protocol for initiating block uploads.
    #[bits(3)]
    pub ccs: u8,

    /// Reserved bits.
    /// These bits are reserved for future use and should typically be set to 0 or left unset.
    #[bits(2)]
    _reserved: u8,

    /// CRC support flag.
    /// If set (`true`), indicates that the SDO block upload will use CRC for ensuring data integrity.
    #[bits(1)]
    pub cc: bool,

    /// Client subcommand.
    /// Specific subcommand for initiating block upload in the SDO protocol.
    #[bits(2)]
    pub cs: u8,
}

/// Represents the SDO (Service Data Object) Block Upload Command.
/// The bitfield representation is based on an `u8` (8-bit unsigned integer) with the most significant bit (MSB) ordering.
#[bitfield(u8, order = Msb)]
pub struct SdoBlockUploadCmd {
    /// Command Specifier.
    /// Indicates the specific command to be executed within the SDO protocol for block uploads.
    #[bits(3)]
    pub ccs: u8,

    /// Reserved bits.
    /// These bits are reserved for future use and should typically be set to 0 or left unset.
    #[bits(3)]
    _reserved: u8,

    /// Client subcommand.
    /// Specific subcommand for block uploads in the SDO protocol.
    #[bits(2)]
    pub cs: u8,
}

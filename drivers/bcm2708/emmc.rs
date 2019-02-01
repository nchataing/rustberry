/*!
 * Provides an interface to the EMMC controller and commands for interacting
 * with an SD card.
 */

/* References:
 *
 * PLSS     - SD Group Physical Layer Simplified Specification ver 3.00
 * HCSS     - SD Group Host Controller Simplified Specification ver 3.00
 *
 * Broadcom BCM2835 Peripherals Guide
 *
 * This code is adapted from
 *  https://github.com/jncronin/rpi-boot/blob/master/emmc.c
 */

use bcm2708;
use mmio;
use uart::{Uart, Write};

const EMMC_BASE: usize = bcm2708::PERIPHERAL_BASE + 0x30_0000;

const EMMC_ARG2: *mut u32 = (EMMC_BASE + 0x00) as *mut u32;
const EMMC_BLKSIZECNT: *mut u32 = (EMMC_BASE + 0x04) as *mut u32;
const EMMC_ARG1: *mut u32 = (EMMC_BASE + 0x08) as *mut u32;
const EMMC_CMDTM: *mut u32 = (EMMC_BASE + 0x0C) as *mut u32;
const EMMC_RESP0: *mut u32 = (EMMC_BASE + 0x10) as *mut u32;
const EMMC_RESP1: *mut u32 = (EMMC_BASE + 0x14) as *mut u32;
const EMMC_RESP2: *mut u32 = (EMMC_BASE + 0x18) as *mut u32;
const EMMC_RESP3: *mut u32 = (EMMC_BASE + 0x1C) as *mut u32;
const EMMC_DATA: *mut u32 = (EMMC_BASE + 0x20) as *mut u32;
const EMMC_STATUS: *mut u32 = (EMMC_BASE + 0x24) as *mut u32;
const EMMC_CONTROL0: *mut u32 = (EMMC_BASE + 0x28) as *mut u32;
const EMMC_CONTROL1: *mut u32 = (EMMC_BASE + 0x2C) as *mut u32;
const EMMC_INTERRUPT: *mut u32 = (EMMC_BASE + 0x30) as *mut u32;
const EMMC_IRPT_MASK: *mut u32 = (EMMC_BASE + 0x34) as *mut u32;
const EMMC_IRPT_EN: *mut u32 = (EMMC_BASE + 0x38) as *mut u32;
const EMMC_CONTROL2: *mut u32 = (EMMC_BASE + 0x3C) as *mut u32;
const EMMC_FORCE_IRPT: *mut u32 = (EMMC_BASE + 0x50) as *mut u32;
const EMMC_BOOT_TIMEOUT: *mut u32 = (EMMC_BASE + 0x70) as *mut u32;
const EMMC_DBG_SEL: *mut u32 = (EMMC_BASE + 0x74) as *mut u32;
const EMMC_EXRDFIFO_CFG: *mut u32 = (EMMC_BASE + 0x80) as *mut u32;
const EMMC_EXRDFIFO_EN: *mut u32 = (EMMC_BASE + 0x84) as *mut u32;
const EMMC_TUNE_STEP: *mut u32 = (EMMC_BASE + 0x88) as *mut u32;
const EMMC_TUNE_STEPS_STD: *mut u32 = (EMMC_BASE + 0x8C) as *mut u32;
const EMMC_TUNE_STEPS_DDR: *mut u32 = (EMMC_BASE + 0x90) as *mut u32;
const EMMC_SPI_INT_SPT: *mut u32 = (EMMC_BASE + 0xF0) as *mut u32;
const EMMC_SLOTISR_VER: *mut u32 = (EMMC_BASE + 0xFC) as *mut u32;

/// Block size
pub const BLOCK_SIZE: usize = 512;

/// EMMC module base clock (100MHz)
const BASE_CLOCK: u32 = 100_000_000;

/// SD Clock Frequencies (in Hz)
enum SdClockFreq {
    Identification = 400_000,
    Normal = 25_000_000,
    High = 50_000_000,
    Sdr50 = 100_000_000,
    Sdr104 = 208_000_000,
}

bitflags! {
    struct SdCmd : u32
    {
        const TYPE_NORMAL = 0b00 << 22;
        const TYPE_SUSPEND = 0b01 << 22;
        const TYPE_RESUME = 0b10 << 22;
        const TYPE_ABORT = 0b11 << 22;
        const TYPE_MASK = 0b11 << 22;

        const ISDATA = 1 << 21;
        const IXCHK_EN = 1 << 20;
        const CRCCHK_EN    = 1 << 19;

        const RSPNS_TYPE_NONE = 0b00 << 16; // For no response
        const RSPNS_TYPE_136 = 0b01 << 16; // For response R2 (with CRC), R3,4 (no CRC)
        const RSPNS_TYPE_48    = 0b10 << 16; // For responses R1, R5, R6, R7 (with CRC)
        const RSPNS_TYPE_48B = 0b11 << 16; // For responses R1b, R5b (with CRC)
        const RSPNS_TYPE_MASK = 0b11 << 16;

        const MULTI_BLOCK = 1 << 5;
        const DAT_DIR_HC = 0 << 4;
        const DAT_DIR_CH = 1 << 4;

        const AUTO_CMD_EN_NONE = 0b00 << 2;
        const AUTO_CMD_EN_CMD12 = 0b01 << 2;
        const AUTO_CMD_EN_CMD23 = 0b10 << 2;
        const AUTO_CMD_EN_MASK = 0b11 << 2;

        const BLKCNT_EN = 1 << 1;

        // Composite flags
        const RESP_NONE = Self::RSPNS_TYPE_NONE.bits;
        const RESP_R1 = Self::RSPNS_TYPE_48.bits | Self::CRCCHK_EN.bits;
        const RESP_R1B = Self::RSPNS_TYPE_48B.bits | Self::CRCCHK_EN.bits;
        const RESP_R2 = Self::RSPNS_TYPE_136.bits | Self::CRCCHK_EN.bits;
        const RESP_R3 = Self::RSPNS_TYPE_48.bits;
        const RESP_R4 = Self::RSPNS_TYPE_136.bits;
        const RESP_R6 = Self::RSPNS_TYPE_48.bits | Self::CRCCHK_EN.bits;
        const RESP_R7 = Self::RSPNS_TYPE_48.bits | Self::CRCCHK_EN.bits;

        const DATA_READ = Self::ISDATA.bits | Self::DAT_DIR_CH.bits;
        const DATA_WRITE = Self::ISDATA.bits | Self::DAT_DIR_HC.bits;

        // Actual commands
        const GO_IDLE_STATE        = 0 << 24;
        const ALL_SEND_CID         = 2 << 24 | Self::RESP_R2.bits;
        const SEND_RELATIVE_ADDR   = 3 << 24 | Self::RESP_R6.bits;
        const SET_DSR              = 4 << 24;
        const IO_SET_OP_COND       = 5 << 24 | Self::RESP_R4.bits;
        const SWITCH_FUNC          = 6 << 24 | Self::RESP_R1.bits;
        const SELECT_DESELECT_CARD = 7 << 24 | Self::RESP_R1B.bits;
        const SEND_IF_COND         = 8 << 24 | Self::RESP_R7.bits;
        const SEND_CSD             = 9 << 24 | Self::RESP_R2.bits;
        const SEND_CID             = 10 << 24 | Self::RESP_R2.bits;
        const VOLTAGE_SWITCH       = 11 << 24 | Self::RESP_R1.bits;
        const STOP_TRANSMISSION    = 12 << 24 | Self::RESP_R1B.bits
                                    | Self::TYPE_ABORT.bits;
        const SEND_STATUS          = 13 << 24 | Self::RESP_R1.bits;
        const GO_INACTIVE_STATE    = 15 << 24;
        const SET_BLOCKLEN         = 16 << 24 | Self::RESP_R1.bits;
        const READ_SINGLE_BLOCK    = 17 << 24 | Self::RESP_R1.bits
                                    | Self::DATA_READ.bits;
        const READ_MULTIPLE_BLOCK  = 18 << 24 | Self::RESP_R1.bits
                                    | Self::DATA_READ.bits
                                    | Self::MULTI_BLOCK.bits
                                    | Self::BLKCNT_EN.bits;
        const SEND_TUNING_BLOCK    = 19 << 24 | Self::RESP_R1.bits
                                    | Self::DATA_READ.bits;
        const SPEED_CLASS_CONTROL  = 20 << 24 | Self::RESP_R1B.bits;
        const SET_BLOCK_COUNT      = 23 << 24 | Self::RESP_R1.bits;
        const WRITE_BLOCK          = 24 << 24 | Self::RESP_R1.bits
                                    | Self::DATA_WRITE.bits;
        const WRITE_MULTIPLE_BLOCK = 25 << 24 | Self::RESP_R1.bits
                                    | Self::DATA_WRITE.bits
                                    | Self::MULTI_BLOCK.bits
                                    | Self::BLKCNT_EN.bits;
        const PROGRAM_CSD          = 27 << 24 | Self::RESP_R1.bits
                                    | Self::DATA_WRITE.bits;
        const SET_WRITE_PROT       = 28 << 24 | Self::RESP_R1B.bits;
        const CLR_WRITE_PROT       = 29 << 24 | Self::RESP_R1B.bits;
        const SEND_WRITE_PROT      = 30 << 24 | Self::RESP_R1.bits
                                    | Self::DATA_READ.bits;
        const ERASE_WR_BLK_START   = 32 << 24 | Self::RESP_R1.bits;
        const ERASE_WR_BLK_END     = 33 << 24 | Self::RESP_R1.bits;
        const ERASE                = 38 << 24 | Self::RESP_R1B.bits;
        const LOCK_UNLOCK          = 42 << 24 | Self::RESP_R1.bits;
        const APP_CMD              = 55 << 24 | Self::RESP_R1.bits;
        const GEN_CMD              = 56 << 24 | Self::RESP_R1.bits
                                    | Self::ISDATA.bits;

        // Application commands
        const IS_APP_CMD = 1 << 31; // This flag must be cleared before sending
        const SET_BUS_WIDTH          = Self::IS_APP_CMD.bits | 6 << 24
                                        | Self::RESP_R1.bits;
        const SD_STATUS              = Self::IS_APP_CMD.bits | 13 << 24
                                        | Self::RESP_R1.bits;
        const SEND_NUM_WR_BLOCKS     = Self::IS_APP_CMD.bits | 22 << 24
                                        | Self::RESP_R1.bits
                                        | Self::DATA_READ.bits;
        const SET_WR_BLK_ERASE_COUNT = Self::IS_APP_CMD.bits | 23 << 24
                                        | Self::RESP_R1.bits;
        const SD_SEND_OP_COND        = Self::IS_APP_CMD.bits | 41 << 24
                                        | Self::RESP_R3.bits;
        const SET_CLR_CARD_DETECT    = Self::IS_APP_CMD.bits | 42 << 24
                                        | Self::RESP_R1.bits;
        const SEND_SCR               = Self::IS_APP_CMD.bits | 51 << 24
                                        | Self::RESP_R1.bits
                                        | Self::DATA_READ.bits;
    }
}

bitflags! {
    pub struct SdInterrupts : u32
    {
        const COMMAND_COMPLETE   = 1 << 0;
        const TRANSFER_COMPLETE  = 1 << 1;
        const BLOCK_GAP_EVENT    = 1 << 2;
        const BUFFER_WRITE_READY = 1 << 4;
        const BUFFER_READ_READY  = 1 << 5;
        const CARD_INTERRUPT     = 1 << 8;

        const ERROR              = 1 << 15;
        const ERR_CMD_TIMEOUT     = 1 << 16;
        const ERR_CMD_CRC         = 1 << 17;
        const ERR_CMD_END_BIT     = 1 << 18;
        const ERR_CMD_INDEX         = 1 << 19;
        const ERR_DATA_TIMEOUT     = 1 << 20;
        const ERR_DATA_CRC         = 1 << 21;
        const ERR_DATA_END_BIT     = 1 << 22;
        const ERR_AUTO_CMD12     = 1 << 24;

        const ERROR_MASK = 0xFFFF_0000;
    }
}

// Control1 flags
bitflags! {
    struct Control1 : u32
    {
        const CLK_INTLEN = 1 << 0;
        const CLK_STABLE = 1 << 1;
        const CLK_EN     = 1 << 2;

        const RESET_ALL = 1 << 24;
        const RESET_CMD = 1 << 25;
        const RESET_DAT = 1 << 26;
    }
}

#[derive(Debug)]
enum SdVersion {
    Ver1,
    Ver1_1,
    Ver2,
    Ver3,
    Ver4,
    Unknown,
}

pub struct SdCard {
    is_sdhc: bool,
    use_1_8v: bool,
    ocr: u16,
    cid: [u32; 4],
    rca: u16,
    sd_version: SdVersion,
    bus_widths: u8,
}

pub struct CmdError {
    issued_command: SdCmd,
    expected_irpt: SdInterrupts,
    error_irpts: SdInterrupts,
}

use core::fmt;
impl fmt::Debug for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}CMD{} recieved {:?} (instead of {:?})",
            if self.issued_command.contains(SdCmd::IS_APP_CMD) {
                "A"
            } else {
                ""
            },
            self.issued_command.bits() >> 24 & 0x7F,
            self.error_irpts,
            self.expected_irpt
        )
    }
}

#[derive(Debug)]
pub enum SdError {
    CommandError(CmdError),
    ControllerVersionUnsupported,
    ControllerResetError,
    ControllerClockError,
    UnusableCard(u32), // Wrong response to CMD8
    SdioCard,
    VoltageSwitchZeroesCheckFailed(u32),
    ControllerDidNotKeepVoltageSwitchSignal,
    VoltageSwitchOnesCheckFailed(u32),
    CardNotReadyForData,
    InvalidCardStatus(u32),
    CmdResetFailure,
    DatResetFailure,
}

impl From<CmdError> for SdError {
    fn from(err: CmdError) -> SdError {
        SdError::CommandError(err)
    }
}

pub fn init() -> Result<SdCard, SdError> {
    // Read the controller version
    let ver = unsafe { mmio::read(EMMC_SLOTISR_VER) };
    //let vendor = ver >> 24;
    let host_version = (ver >> 16) & 0xff;
    //let slot_status = ver & 0xff;

    if host_version < 2 {
        return Err(SdError::ControllerVersionUnsupported);
    }

    // Reset the controller
    unsafe {
        let mut control1 = mmio::read(EMMC_CONTROL1);
        control1 |= Control1::RESET_ALL.bits();
        control1 &= !(Control1::CLK_INTLEN | Control1::CLK_EN).bits();
        mmio::write(EMMC_CONTROL1, control1);

        if timeout_wait_while!(mmio::read(EMMC_CONTROL1) & (0b111 << 24) != 0, 10_000_000) {
            return Err(SdError::ControllerResetError);
        }

        // Clear control2
        mmio::write(EMMC_CONTROL2, 0);

        control1 = mmio::read(EMMC_CONTROL1);
        control1 |= Control1::CLK_INTLEN.bits();
        let id_freq_divider = get_clock_divider(SdClockFreq::Identification);
        control1 |= id_freq_divider;
        control1 |= 0b0111 << 16; // data timeout = TMCLK * 2^10
        mmio::write(EMMC_CONTROL1, control1);

        if timeout_wait_while!(mmio::read(EMMC_CONTROL1) & (1 << 1) == 0, 10_000_000) {
            return Err(SdError::ControllerClockError);
        }

        // Enable the SD clock
        control1 = mmio::read(EMMC_CONTROL1);
        control1 |= 1 << 2;
        mmio::write(EMMC_CONTROL1, control1);
        mmio::delay(0x2000);

        // Mask off sending interrupts to the ARM
        mmio::write(EMMC_IRPT_EN, 0);
        // Reset interrupts
        mmio::write(EMMC_INTERRUPT, 0xffff_ffff);
        // Have all interrupts sent to the INTERRUPT register
        mmio::write(EMMC_IRPT_MASK, !SdInterrupts::CARD_INTERRUPT.bits());

        // Send CMD0 (reset to idle state) to the card
        issue_simple_cmd(SdCmd::GO_IDLE_STATE, 0)?;

        // Send CMD8 to the card
        // Voltage supplied = 0x1 = 2.7-3.6V (standard)
        // Check pattern = 10101010b (as per PLSS 4.3.13) = 0xAA
        // A timeout error on the following command (CMD8) is normal
        // and expected if the SD card version is less than 2.0
        let v2support;
        match issue_simple_cmd(SdCmd::SEND_IF_COND, 0x1AA) {
            Ok(resp) => {
                if resp.r48 & 0xFFF != 0x1AA {
                    return Err(SdError::UnusableCard(resp.r48));
                }
                v2support = true;
            }
            Err(ref e) if e.error_irpts == SdInterrupts::ERR_CMD_TIMEOUT => {
                reset_cmd()?;
                v2support = false;
            }
            Err(e) => return Err(SdError::CommandError(e)),
        }

        // Here we are supposed to check the response to CMD5 (HCSS 3.6)
        // It only returns if the card is a SDIO card
        match issue_simple_cmd(SdCmd::IO_SET_OP_COND, 0) {
            Err(ref e) if e.error_irpts == SdInterrupts::ERR_CMD_TIMEOUT => reset_cmd()?,
            Err(e) => return Err(SdError::CommandError(e)),
            _ => return Err(SdError::SdioCard),
        }

        let ocr;
        let is_sdhc;
        let supports_1_8v;
        loop {
            let v2flags = if v2support { 1 << 30 | 1 << 24 } else { 0 };
            let op_cond = issue_simple_acmd(SdCmd::SD_SEND_OP_COND, 0x00FF_8000 | v2flags, 0)?.r48;

            if op_cond & (1 << 31) != 0 {
                ocr = (op_cond >> 8) as u16;
                is_sdhc = op_cond & 1 << 30 != 0;
                supports_1_8v = op_cond & 1 << 24 != 0;

                break;
            } else {
                mmio::delay(5_000_000);
            }
        }

        switch_clock_rate(SdClockFreq::Normal);
        mmio::delay(50_000);

        // Switch to 1.8V mode if possible
        if supports_1_8v {
            voltage_switch()?;
        }

        // Send CMD2 to get the cards CID
        let cid = issue_simple_cmd(SdCmd::ALL_SEND_CID, 0)?.r136;

        // Send CMD3 to enter the data state
        let cmd3_resp = issue_simple_cmd(SdCmd::SEND_RELATIVE_ADDR, 0)?.r48;
        let rca = (cmd3_resp >> 16) as u16;

        if (cmd3_resp & 0x7000 != 0) || (cmd3_resp & 0x100 == 0) {
            return Err(SdError::CardNotReadyForData);
        }

        // Now select the card (toggles it to transfer state)
        let cmd7_resp = issue_busy_cmd(SdCmd::SELECT_DESELECT_CARD, (rca as u32) << 16)?.r48;
        let status = (cmd7_resp >> 9) & 0xF;
        if status != 3 && status != 4 {
            return Err(SdError::InvalidCardStatus(status));
        }

        // If not an SDHC card, ensure BLOCKLEN is 512 bytes
        if !is_sdhc {
            issue_simple_cmd(SdCmd::SET_BLOCKLEN, 512)?;
        }

        // Get the cards SCR register
        let mut scr = [0; 2];
        issue_read_acmd(SdCmd::SEND_SCR, 0, rca, &mut scr, 8)?;

        let scr0 = u32::from_be(scr[0]);
        let sd_spec = (scr0 >> (56 - 32)) & 0xf;
        let sd_spec3 = (scr0 >> (47 - 32)) & 0x1;
        let sd_spec4 = (scr0 >> (42 - 32)) & 0x1;

        let bus_widths = ((scr0 >> (48 - 32)) & 0xf) as u8;
        let sd_version;
        if sd_spec == 0 {
            sd_version = SdVersion::Ver1
        } else if sd_spec == 1 {
            sd_version = SdVersion::Ver1_1
        } else if sd_spec == 2 {
            if sd_spec3 == 0 {
                sd_version = SdVersion::Ver2
            } else {
                if sd_spec4 == 0 {
                    sd_version = SdVersion::Ver3
                } else {
                    sd_version = SdVersion::Ver4
                }
            }
        } else {
            sd_version = SdVersion::Unknown
        }

        if bus_widths & 0x4 != 0 {
            switch_to_4bit_data(rca)?;
        }

        Ok(SdCard {
            is_sdhc,
            use_1_8v: supports_1_8v,
            ocr,
            cid,
            rca,
            sd_version,
            bus_widths,
        })
    }
}

unsafe fn voltage_switch() -> Result<(), SdError> {
    // As per HCSS 3.6.1

    // Send VOLTAGE_SWITCH
    issue_simple_cmd(SdCmd::VOLTAGE_SWITCH, 0)?;

    // Disable SD clock
    let control1 = mmio::read(EMMC_CONTROL1) & !Control1::CLK_EN.bits();
    mmio::write(EMMC_CONTROL1, control1);

    // Check DAT[3:0]
    let status_reg = mmio::read(EMMC_STATUS);
    let dat = (status_reg >> 20) & 0xf;
    if dat != 0 {
        return Err(SdError::VoltageSwitchZeroesCheckFailed(dat));
    }

    // Set 1.8V signal enable to 1
    let control0 = mmio::read(EMMC_CONTROL0) | (1 << 8);
    mmio::write(EMMC_CONTROL0, control0);

    // Wait ~5 ms
    mmio::delay(5_000_000);

    // Check the 1.8V signal enable is set
    let control0 = mmio::read(EMMC_CONTROL0);
    if ((control0 >> 8) & 0x1) == 0 {
        return Err(SdError::ControllerDidNotKeepVoltageSwitchSignal);
    }

    // Re-enable the SD clock
    let control1 = mmio::read(EMMC_CONTROL1) | Control1::CLK_EN.bits();
    mmio::write(EMMC_CONTROL1, control1);

    // Wait ~1 ms
    mmio::delay(1_000_000);

    // Check DAT[3:0]
    let status_reg = mmio::read(EMMC_STATUS);
    let dat = (status_reg >> 20) & 0xf;
    if dat != 0b1111 {
        return Err(SdError::VoltageSwitchOnesCheckFailed(dat));
    }

    Ok(())
}

unsafe fn switch_to_4bit_data(rca: u16) -> Result<(), CmdError> {
    // Set 4-bit transfer mode (ACMD6)
    // See HCSS 3.4 for the algorithm

    // Disable card interrupt in host
    let old_irpt_mask = mmio::read(EMMC_IRPT_MASK);
    let new_iprt_mask = old_irpt_mask & !SdInterrupts::CARD_INTERRUPT.bits();
    mmio::write(EMMC_IRPT_MASK, new_iprt_mask);

    // Send ACMD6 to change the card's bit mode
    issue_simple_acmd(SdCmd::SET_BUS_WIDTH, 0x2, rca)?;

    // Change bit mode for Host
    let control0 = mmio::read(EMMC_CONTROL0) | 0x2;
    mmio::write(EMMC_CONTROL0, control0);

    // Re-enable card interrupt in host
    mmio::write(EMMC_IRPT_MASK, old_irpt_mask);

    Ok(())
}

union CmdResponse {
    empty: (),
    r48: u32,
    r136: [u32; 4],
}

unsafe fn issue_cmd_base(cmd: SdCmd, arg: u32) -> Result<CmdResponse, CmdError> {
    assert!(!cmd.contains(SdCmd::IS_APP_CMD));

    // Wait for Command Inhibit to go down
    while mmio::read(EMMC_STATUS) & 0x1 != 0 {}

    mmio::write(EMMC_ARG1, arg);
    mmio::write(EMMC_CMDTM, cmd.bits());

    wait_interrupt(SdInterrupts::COMMAND_COMPLETE).map_err(|e| CmdError {
        issued_command: cmd,
        ..e
    })?;

    match cmd & SdCmd::RSPNS_TYPE_MASK {
        SdCmd::RSPNS_TYPE_48 | SdCmd::RSPNS_TYPE_48B => Ok(CmdResponse {
            r48: mmio::read(EMMC_RESP0),
        }),
        SdCmd::RSPNS_TYPE_136 => Ok(CmdResponse {
            r136: [
                mmio::read(EMMC_RESP0),
                mmio::read(EMMC_RESP1),
                mmio::read(EMMC_RESP2),
                mmio::read(EMMC_RESP3),
            ],
        }),
        _ => Ok(CmdResponse { empty: () }),
    }
}

unsafe fn issue_simple_cmd(cmd: SdCmd, arg: u32) -> Result<CmdResponse, CmdError> {
    assert!(!cmd.contains(SdCmd::ISDATA));
    assert!(cmd & SdCmd::RSPNS_TYPE_MASK != SdCmd::RSPNS_TYPE_48B);

    clean_interrupts();
    issue_cmd_base(cmd, arg)
}

unsafe fn issue_busy_cmd(cmd: SdCmd, arg: u32) -> Result<CmdResponse, CmdError> {
    assert!(!cmd.contains(SdCmd::ISDATA));
    assert!(cmd & SdCmd::RSPNS_TYPE_MASK == SdCmd::RSPNS_TYPE_48B);

    clean_interrupts();

    if cmd & SdCmd::TYPE_MASK != SdCmd::TYPE_ABORT {
        // Not an abort command: wait for the data line to be free
        while mmio::read(EMMC_STATUS) & 0x2 != 0 {}
    }

    let response = issue_cmd_base(cmd, arg)?;
    wait_for_transfer_complete().map_err(|e| CmdError {
        issued_command: cmd,
        ..e
    })?;
    Ok(response)
}

unsafe fn issue_read_cmd(
    cmd: SdCmd,
    arg: u32,
    buffer: &mut [u32],
    block_size: usize,
) -> Result<CmdResponse, CmdError> {
    assert!(cmd.contains(SdCmd::DATA_READ));
    assert!((buffer.len() * 4) % block_size == 0);

    let block_count = buffer.len() * 4 / block_size;
    assert!(block_count <= 0xffff);

    let blksizecnt = block_size | (block_count << 16);
    mmio::write(EMMC_BLKSIZECNT, blksizecnt as u32);

    clean_interrupts();
    let response = issue_cmd_base(cmd, arg)?;

    for block in buffer.exact_chunks_mut(block_size / 4) {
        wait_interrupt(SdInterrupts::BUFFER_READ_READY).map_err(|e| CmdError {
            issued_command: cmd,
            ..e
        })?;

        // Read the block
        for word in block {
            *word = mmio::read(EMMC_DATA);
        }
    }

    wait_for_transfer_complete().map_err(|e| CmdError {
        issued_command: cmd,
        ..e
    })?;
    Ok(response)
}

unsafe fn issue_write_cmd(
    cmd: SdCmd,
    arg: u32,
    buffer: &[u32],
    block_size: usize,
) -> Result<CmdResponse, CmdError> {
    assert!(cmd.contains(SdCmd::DATA_WRITE));
    assert!((buffer.len() * 4) % block_size == 0);

    let block_count = buffer.len() * 4 / block_size;
    assert!(block_count <= 0xffff);

    let blksizecnt = block_size | (block_count << 16);
    mmio::write(EMMC_BLKSIZECNT, blksizecnt as u32);

    clean_interrupts();
    let response = issue_cmd_base(cmd, arg)?;

    for block in buffer.exact_chunks(block_size / 4) {
        wait_interrupt(SdInterrupts::BUFFER_WRITE_READY).map_err(|e| CmdError {
            issued_command: cmd,
            ..e
        })?;

        // Read the block
        for word in block {
            mmio::write(EMMC_DATA, *word);
        }
    }

    wait_for_transfer_complete().map_err(|e| CmdError {
        issued_command: cmd,
        ..e
    })?;
    Ok(response)
}

unsafe fn issue_simple_acmd(cmd: SdCmd, arg: u32, rca: u16) -> Result<CmdResponse, CmdError> {
    assert!(cmd.contains(SdCmd::IS_APP_CMD));
    assert!(!cmd.contains(SdCmd::ISDATA));
    assert!(cmd & SdCmd::RSPNS_TYPE_MASK != SdCmd::RSPNS_TYPE_48B);

    clean_interrupts();
    issue_cmd_base(SdCmd::APP_CMD, (rca as u32) << 16)?;
    issue_cmd_base(cmd & !(SdCmd::IS_APP_CMD), arg).map_err(|e| CmdError {
        issued_command: cmd,
        ..e
    })
}

unsafe fn issue_read_acmd(
    cmd: SdCmd,
    arg: u32,
    rca: u16,
    buffer: &mut [u32],
    block_size: usize,
) -> Result<CmdResponse, CmdError> {
    assert!(cmd.contains(SdCmd::IS_APP_CMD));

    clean_interrupts();
    issue_cmd_base(SdCmd::APP_CMD, (rca as u32) << 16)?;
    issue_read_cmd(cmd & !(SdCmd::IS_APP_CMD), arg, buffer, block_size).map_err(|e| CmdError {
        issued_command: cmd,
        ..e
    })
}

unsafe fn wait_interrupt(wanted_irpt: SdInterrupts) -> Result<(), CmdError> {
    let mask = (wanted_irpt | SdInterrupts::ERROR).bits();
    while mmio::read(EMMC_INTERRUPT) & mask == 0 {}

    let irpts = SdInterrupts::from_bits_truncate(mmio::read(EMMC_INTERRUPT));
    let masked_irpts = irpts & (SdInterrupts::ERROR_MASK | wanted_irpt);
    mmio::write(EMMC_INTERRUPT, masked_irpts.bits());

    if masked_irpts == wanted_irpt {
        Ok(())
    } else {
        Err(CmdError {
            issued_command: SdCmd::empty(),
            expected_irpt: wanted_irpt,
            error_irpts: masked_irpts,
        })
    }
}

unsafe fn clean_interrupts() {
    let irpts = SdInterrupts::from_bits_truncate(mmio::read(EMMC_INTERRUPT));
    if irpts != SdInterrupts::empty() {
        write!(Uart, "SD: spurious interrupts found: {:?}", irpts).unwrap();
    }
    mmio::write(EMMC_INTERRUPT, irpts.bits());
}

/// Reset the CMD line
unsafe fn reset_cmd() -> Result<(), SdError> {
    let control1 = mmio::read(EMMC_CONTROL1) | Control1::RESET_CMD.bits();
    mmio::write(EMMC_CONTROL1, control1);

    if timeout_wait_while!(
        mmio::read(EMMC_CONTROL1) & Control1::RESET_CMD.bits() != 0,
        100_000_000
    ) {
        return Err(SdError::CmdResetFailure);
    }

    Ok(())
}

/// Reset the DAT line
unsafe fn reset_dat() -> Result<(), SdError> {
    let control1 = mmio::read(EMMC_CONTROL1) | Control1::RESET_DAT.bits();
    mmio::write(EMMC_CONTROL1, control1);

    if timeout_wait_while!(
        mmio::read(EMMC_CONTROL1) & Control1::RESET_DAT.bits() != 0,
        100_000_000
    ) {
        return Err(SdError::DatResetFailure);
    }

    Ok(())
}

unsafe fn wait_for_transfer_complete() -> Result<(), CmdError> {
    // First check command inhibit (DAT) is not already 0
    if mmio::read(EMMC_STATUS) & 0b10 == 0 {
        mmio::write(EMMC_INTERRUPT, 0xffff0002);
    } else {
        let wait_result = wait_interrupt(SdInterrupts::TRANSFER_COMPLETE);

        // Handle the case where both data timeout and transfer complete
        // are set - transfer complete overrides data timeout: HCSS 2.2.17
        match wait_result {
            Ok(()) => (),
            Err(ref e)
                if e.error_irpts
                    == SdInterrupts::TRANSFER_COMPLETE | SdInterrupts::ERR_DATA_TIMEOUT =>
            {
                ()
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Give the clock divider flags of EMMC_CONTROL1 to reach the target_freq
fn get_clock_divider(target_freq: SdClockFreq) -> u32 {
    let target_rate = target_freq as u32;
    let mut targetted_divisor;
    if target_rate > BASE_CLOCK {
        targetted_divisor = 1;
    } else {
        targetted_divisor = BASE_CLOCK / target_rate;
        if BASE_CLOCK % target_rate != 0 {
            targetted_divisor -= 1;
        }
    }

    // Decide on the clock mode to use

    // Currently only 10-bit divided clock mode is supported

    // HCI version 3 or greater supports 10-bit divided clock mode
    // This requires a power-of-two divider

    // Find the first bit set
    let mut divisor = 31;
    for first_bit in (0..32).rev() {
        let bit_test = 1 << first_bit;
        if targetted_divisor & bit_test != 0 {
            divisor = first_bit;
            targetted_divisor &= !bit_test;
            if targetted_divisor != 0 {
                // The divisor is not a power-of-two, increase it
                divisor += 1;
            }
            break;
        }
    }

    if divisor >= 32 {
        divisor = 31;
    }

    if divisor != 0 {
        divisor = 1 << (divisor - 1);
    }

    if divisor >= 0x400 {
        divisor = 0x3ff;
    }

    let freq_select = divisor & 0xff;
    let upper_bits = (divisor >> 8) & 0x3;
    (freq_select << 8) | (upper_bits << 6) | (0 << 5)
}

/// Switch the clock frequency whilst running
unsafe fn switch_clock_rate(target_freq: SdClockFreq) {
    // Decide on an appropriate divider
    let divider = get_clock_divider(target_freq);

    // Wait for the command inhibit (CMD and DAT) bits to clear
    while mmio::read(EMMC_STATUS) & 0b11 != 0 {}

    // Set the SD clock off
    let mut control1 = mmio::read(EMMC_CONTROL1);
    control1 &= !Control1::CLK_EN.bits();
    mmio::write(EMMC_CONTROL1, control1);
    mmio::delay(0x2000);

    // Write the new divider
    control1 &= !0xffe0; // Clear old setting + clock generator select
    control1 |= divider;
    mmio::write(EMMC_CONTROL1, control1);
    mmio::delay(0x2000);

    // Enable the SD clock
    control1 |= Control1::CLK_EN.bits();
    mmio::write(EMMC_CONTROL1, control1);
    mmio::delay(0x2000);
}

impl SdCard {
    unsafe fn ensure_data_mode(&self) -> Result<(), SdError> {
        let status = issue_simple_cmd(SdCmd::SEND_STATUS, (self.rca as u32) << 16)?.r48;
        let cur_state = (status >> 9) & 0xf;

        if cur_state == 3 {
            // Currently in the stand-by state - select it
            issue_simple_cmd(SdCmd::SELECT_DESELECT_CARD, (self.rca as u32) << 16)?;
        } else if cur_state == 5 {
            // In the data transfer state - cancel the transmission
            issue_simple_cmd(SdCmd::STOP_TRANSMISSION, 0)?;
            reset_dat()?;
        }

        // Check again that we're now in the correct mode
        if cur_state != 4 {
            let new_status = issue_simple_cmd(SdCmd::SEND_STATUS, (self.rca as u32) << 16)?.r48;
            let new_state = (new_status >> 9) & 0xf;

            if new_state != 4 {
                return Err(SdError::InvalidCardStatus(new_state));
            }
        }

        Ok(())
    }

    pub fn read(&self, data: &mut [u8], block_id: usize) -> Result<(), SdError> {
        let block_addr;
        if self.is_sdhc {
            block_addr = block_id as u32;
        } else {
            block_addr = (block_id * BLOCK_SIZE) as u32;
        }

        let command;
        if data.len() == BLOCK_SIZE {
            command = SdCmd::READ_SINGLE_BLOCK;
        } else {
            command = SdCmd::READ_MULTIPLE_BLOCK;
        }

        unsafe {
            self.ensure_data_mode()?;

            let buffer =
                ::core::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u32, data.len() / 4);
            issue_read_cmd(command, block_addr, buffer, BLOCK_SIZE)?;
        }

        Ok(())
    }

    pub fn write(&self, data: &[u8], block_id: usize) -> Result<(), SdError> {
        let block_addr;
        if self.is_sdhc {
            block_addr = block_id as u32;
        } else {
            block_addr = (block_id * BLOCK_SIZE) as u32;
        }

        let command;
        if data.len() == BLOCK_SIZE {
            command = SdCmd::WRITE_BLOCK;
        } else {
            command = SdCmd::WRITE_MULTIPLE_BLOCK;
        }

        unsafe {
            self.ensure_data_mode()?;

            let buffer = ::core::slice::from_raw_parts(data.as_ptr() as *const u32, data.len() / 4);
            issue_write_cmd(command, block_addr, buffer, BLOCK_SIZE)?;
        }

        Ok(())
    }
}

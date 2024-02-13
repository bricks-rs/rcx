use crate::opcodes::GetVersionsResponse;
use std::fmt::{self, Display, Formatter};

impl Display for GetVersionsResponse {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        // versions are reverse-endian hex
        let rom_maj = self.rom[0].to_be();
        let rom_min = self.rom[1].to_be();
        let fw_maj = self.firmware[0].to_be();
        let fw_min = self.firmware[1].to_be();
        write!(
            fmt,
            "ROM: {rom_maj:x}.{rom_min:x}; FW: {fw_maj:x}.{fw_min:x}",
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn versions() {
        let v = GetVersionsResponse {
            rom: [768, 256],
            firmware: [768, 515],
        };
        assert_eq!(v.to_string(), "ROM: 3.1; FW: 3.302");
    }
}

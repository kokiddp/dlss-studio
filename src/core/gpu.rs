use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub dedicated_video_memory: usize,
    pub is_rtx_40: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvidiaArch {
    Turing,
    Ampere,
    Ada,
    Blackwell,
    Unknown,
}

impl GpuInfo {
    pub fn nvidia_arch(&self) -> NvidiaArch {
        nvidia_arch(&self.name, self.vendor_id, self.device_id)
    }
}

/// Fail closed for unknown/workstation names; never infer an RTX 30 card from
/// an arbitrary PCI ID range (Ampere also includes non-SM86 compute devices).
pub fn nvidia_arch(name: &str, vendor_id: u32, _device_id: u32) -> NvidiaArch {
    if vendor_id != 0x10de {
        return NvidiaArch::Unknown;
    }
    let upper = name.to_ascii_uppercase();
    let model = upper.split("GEFORCE RTX ").nth(1)
        .and_then(|s| s.split_whitespace().next())
        .unwrap_or("");
    if model.len() != 4 || !model.bytes().all(|b| b.is_ascii_digit()) {
        return NvidiaArch::Unknown;
    }
    match &model[..2] {
        "20" => NvidiaArch::Turing,
        "30" => NvidiaArch::Ampere,
        "40" => NvidiaArch::Ada,
        "50" => NvidiaArch::Blackwell,
        _ => NvidiaArch::Unknown,
    }
}

pub fn detect_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    unsafe {
        let factory: Result<IDXGIFactory1, _> = CreateDXGIFactory1();
        if let Ok(factory) = factory {
            let mut index = 0;
            while let Ok(adapter) = factory.EnumAdapters1(index) {
                if let Ok(desc) = adapter.GetDesc1() {
                    let len = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
                    let name = String::from_utf16_lossy(&desc.Description[..len]);

                    // Ignore Microsoft Basic Render Driver
                    if desc.VendorId != 0x1414 {
                        let is_rtx_40 = is_ada_lovelace(&name, desc.VendorId, desc.DeviceId);
                        gpus.push(GpuInfo {
                            name,
                            vendor_id: desc.VendorId,
                            device_id: desc.DeviceId,
                            dedicated_video_memory: desc.DedicatedVideoMemory,
                            is_rtx_40,
                        });
                    }
                }
                index += 1;
            }
        }
    }

    gpus
}

pub fn is_ada_lovelace(name: &str, vendor_id: u32, device_id: u32) -> bool {
    nvidia_arch(name, vendor_id, device_id) == NvidiaArch::Ada
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_geforce_but_not_workstation_or_other_vendor() {
        for (name, arch) in [
            ("NVIDIA GeForce RTX 2080 Ti", NvidiaArch::Turing),
            ("NVIDIA GeForce RTX 3080 Ti", NvidiaArch::Ampere),
            ("NVIDIA GeForce RTX 3070 Laptop GPU", NvidiaArch::Ampere),
            ("NVIDIA GeForce RTX 4090", NvidiaArch::Ada),
            ("NVIDIA GeForce RTX 5070", NvidiaArch::Blackwell),
            ("NVIDIA RTX A3000", NvidiaArch::Unknown),
            ("NVIDIA Quadro RTX 4000", NvidiaArch::Unknown),
        ] {
            assert_eq!(nvidia_arch(name, 0x10de, 0), arch);
            assert_eq!(nvidia_arch(name, 0x1002, 0), NvidiaArch::Unknown);
        }
    }
}

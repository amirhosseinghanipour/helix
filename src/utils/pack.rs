use anyhow::{Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackHeader {
    pub signature: [u8; 4], // "PACK"
    pub version: u32,
    pub object_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackObject {
    pub object_type: u8,
    pub size: u64,
    pub data: Vec<u8>,
    pub delta_base: Option<String>, // For delta objects
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub header: PackHeader,
    pub objects: Vec<PackObject>,
    pub index: HashMap<String, usize>, // hash -> object index
}

impl Pack {
    pub fn new() -> Self {
        Self {
            header: PackHeader {
                signature: *b"PACK",
                version: 2,
                object_count: 0,
            },
            objects: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn add_object(&mut self, hash: &str, object_type: u8, data: Vec<u8>) {
        let object = PackObject {
            object_type,
            size: data.len() as u64,
            data,
            delta_base: None,
        };
        
        self.index.insert(hash.to_string(), self.objects.len());
        self.objects.push(object);
        self.header.object_count = self.objects.len() as u32;
    }

    pub fn add_delta_object(&mut self, hash: &str, object_type: u8, data: Vec<u8>, base_hash: &str) {
        let object = PackObject {
            object_type,
            size: data.len() as u64,
            data,
            delta_base: Some(base_hash.to_string()),
        };
        
        self.index.insert(hash.to_string(), self.objects.len());
        self.objects.push(object);
        self.header.object_count = self.objects.len() as u32;
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        
        // Write header
        buffer.extend_from_slice(&self.header.signature);
        buffer.extend_from_slice(&self.header.version.to_be_bytes());
        buffer.extend_from_slice(&self.header.object_count.to_be_bytes());
        
        // Write objects
        for object in &self.objects {
            // Write object header
            let header_byte = (object.object_type << 4) | (object.size & 0x0F) as u8;
            buffer.push(header_byte);
            
            if object.size >= 0x0F {
                let mut size = object.size >> 4;
                while size > 0 {
                    let byte = (size & 0x7F) as u8;
                    size >>= 7;
                    if size > 0 {
                        buffer.push(byte | 0x80);
                    } else {
                        buffer.push(byte);
                    }
                }
            }
            
            // Write object data
            buffer.extend_from_slice(&object.data);
        }
        
        Ok(buffer)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid pack data: too short"));
        }
        
        let signature = [data[0], data[1], data[2], data[3]];
        if signature != *b"PACK" {
            return Err(anyhow::anyhow!("Invalid pack signature"));
        }
        
        let version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let object_count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        
        let mut pack = Pack {
            header: PackHeader {
                signature,
                version,
                object_count,
            },
            objects: Vec::new(),
            index: HashMap::new(),
        };
        
        let mut offset = 12;
        for i in 0..object_count {
            let (object, new_offset) = Self::parse_object(&data[offset..])?;
            pack.index.insert(format!("object_{}", i), pack.objects.len());
            pack.objects.push(object);
            offset += new_offset;
        }
        
        Ok(pack)
    }

    fn parse_object(data: &[u8]) -> Result<(PackObject, usize)> {
        let mut offset = 0;
        
        // Parse object header
        let header_byte = data[offset];
        offset += 1;
        
        let object_type = (header_byte >> 4) & 0x07;
        let mut size = (header_byte & 0x0F) as u64;
        
        if (header_byte & 0x80) != 0 {
            let mut shift = 4;
            loop {
                if offset >= data.len() {
                    return Err(anyhow::anyhow!("Invalid pack object header"));
                }
                let byte = data[offset];
                offset += 1;
                size |= ((byte & 0x7F) as u64) << shift;
                if (byte & 0x80) == 0 {
                    break;
                }
                shift += 7;
            }
        }
        
        // For now, we'll assume the rest is object data
        // In a real implementation, you'd need to handle compression
        let object_data = data[offset..offset + size as usize].to_vec();
        offset += size as usize;
        
        let object = PackObject {
            object_type,
            size,
            data: object_data,
            delta_base: None,
        };
        
        Ok((object, offset))
    }
}

pub struct PackBuilder {
    objects: HashMap<String, Vec<u8>>,
    deltas: HashMap<String, (String, Vec<u8>)>, // hash -> (base_hash, delta_data)
}

impl PackBuilder {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            deltas: HashMap::new(),
        }
    }

    pub fn add_object(&mut self, hash: &str, data: Vec<u8>) {
        self.objects.insert(hash.to_string(), data);
    }

    pub fn create_delta(&mut self, hash: &str, base_hash: &str, new_data: &[u8]) -> Result<()> {
        if let Some(base_data) = self.objects.get(base_hash) {
            let delta = self.compute_delta(base_data, new_data)?;
            self.deltas.insert(hash.to_string(), (base_hash.to_string(), delta));
        }
        Ok(())
    }

    /// Compute a real delta between base and target using a simple diff algorithm.
    /// The delta format is:
    /// [base_size (u32)][target_size (u32)][series of (copy/insert) instructions]
    /// Each instruction is:
    ///   - 0x00 [offset (u32)][length (u32)]  -- copy from base
    ///   - 0x01 [length (u32)][data ...]      -- insert new data
    fn compute_delta(&self, base: &[u8], target: &[u8]) -> Result<Vec<u8>> {
        use std::cmp::min;

        let mut delta = Vec::new();
        // Write base and target sizes
        delta.extend_from_slice(&(base.len() as u32).to_be_bytes());
        delta.extend_from_slice(&(target.len() as u32).to_be_bytes());

        let mut base_pos = 0;
        let mut target_pos = 0;

        while target_pos < target.len() {
            // Find the longest match of base in target starting at target_pos
            let mut best_base_off = 0;
            let mut best_len = 0;

            for b_off in 0..base.len() {
                let mut l = 0;
                while b_off + l < base.len()
                    && target_pos + l < target.len()
                    && base[b_off + l] == target[target_pos + l]
                {
                    l += 1;
                }
                if l > best_len && l >= 8 {
                    // Only consider matches of length >= 8 for efficiency
                    best_base_off = b_off;
                    best_len = l;
                }
            }

            if best_len >= 8 {
                // Emit copy instruction
                delta.push(0x00);
                delta.extend_from_slice(&(best_base_off as u32).to_be_bytes());
                delta.extend_from_slice(&(best_len as u32).to_be_bytes());
                target_pos += best_len;
            } else {
                // Emit insert instruction for a single byte (or run of non-matching bytes)
                let mut insert_start = target_pos;
                let mut insert_len = 1;
                while target_pos + insert_len < target.len() {
                    // Try to extend insert run until a match of >=8 is found
                    let mut found = false;
                    for b_off in 0..base.len() {
                        let mut l = 0;
                        while b_off + l < base.len()
                            && target_pos + insert_len + l < target.len()
                            && base[b_off + l] == target[target_pos + insert_len + l]
                        {
                            l += 1;
                        }
                        if l >= 8 {
                            found = true;
                            break;
                        }
                    }
                    if found {
                        break;
                    }
                    insert_len += 1;
                }
                delta.push(0x01);
                delta.extend_from_slice(&(insert_len as u32).to_be_bytes());
                delta.extend_from_slice(&target[insert_start..insert_start + insert_len]);
                target_pos += insert_len;
            }
        }

        Ok(delta)
    }

    pub fn build_pack(&self) -> Pack {
        let mut pack = Pack::new();
        
        // Add all objects
        for (hash, data) in &self.objects {
            pack.add_object(hash, 1, data.clone()); // Assume type 1 (commit) for now
        }
        
        // Add delta objects
        for (hash, (base_hash, delta_data)) in &self.deltas {
            pack.add_delta_object(hash, 7, delta_data.clone(), base_hash); // Type 7 for delta
        }
        
        pack
    }
}

pub fn create_thin_pack(
    local_objects: &HashMap<String, Vec<u8>>,
    remote_objects: &HashMap<String, Vec<u8>>,
) -> Pack {
    let mut pack = Pack::new();
    
    for (hash, data) in local_objects {
        if !remote_objects.contains_key(hash) {
            pack.add_object(hash, 1, data.clone());
        }
    }
    
    pack
}

pub fn extract_objects_from_pack(pack: &Pack) -> HashMap<String, Vec<u8>> {
    let mut objects = HashMap::new();
    
    for (hash, &index) in &pack.index {
        if let Some(object) = pack.objects.get(index) {
            objects.insert(hash.clone(), object.data.clone());
        }
    }
    
    objects
} 
// Vector quantization implementation for storage efficiency

use crate::vectordb::types::Vector;
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

/// Quantization method for vectors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationMethod {
    /// No quantization (raw f32 vectors)
    None,
    /// Simple scalar quantization (8-bit per dimension)
    Scalar8Bit,
    /// Product quantization (future)
    ProductQuantization,
}

impl Default for QuantizationMethod {
    fn default() -> Self {
        Self::None
    }
}

/// Vector quantization to reduce storage requirements
pub struct VectorQuantizer {
    /// Quantization method in use
    method: QuantizationMethod,
    /// Vector dimension
    dimension: usize,
    /// Minimum values per dimension (for scalar quantization)
    min_values: Option<Vec<f32>>,
    /// Maximum values per dimension (for scalar quantization)
    max_values: Option<Vec<f32>>,
    /// Cache of already quantized vectors
    quantization_cache: HashMap<String, Vec<u8>>,
}

impl VectorQuantizer {
    /// Create a new quantizer without initialization
    pub fn new(method: QuantizationMethod, dimension: usize) -> Self {
        Self {
            method,
            dimension,
            min_values: None,
            max_values: None,
            quantization_cache: HashMap::new(),
        }
    }

    /// Initialize the quantizer with a set of vectors (required for scalar quantization)
    pub fn initialize(&mut self, vectors: &[Vector]) -> Result<()> {
        // Skip initialization for None quantization
        if self.method == QuantizationMethod::None {
            return Ok(());
        }

        // Validate vectors
        if vectors.is_empty() {
            anyhow::bail!("Cannot initialize quantizer with empty vector set");
        }

        // Check if all vectors have the same dimension
        for vector in vectors {
            if vector.dimension() != self.dimension {
                anyhow::bail!(
                    "Vector dimension mismatch: expected {}, got {}",
                    self.dimension,
                    vector.dimension()
                );
            }
        }

        match self.method {
            QuantizationMethod::None => {
                // No initialization needed
                Ok(())
            }
            QuantizationMethod::Scalar8Bit => {
                // Find min/max values for each dimension
                let mut min_values = vec![f32::MAX; self.dimension];
                let mut max_values = vec![f32::MIN; self.dimension];

                for vector in vectors {
                    for (i, &val) in vector.values.iter().enumerate() {
                        min_values[i] = min_values[i].min(val);
                        max_values[i] = max_values[i].max(val);
                    }
                }

                // Adjust min/max to avoid division by zero
                for i in 0..self.dimension {
                    if (max_values[i] - min_values[i]).abs() < 1e-6 {
                        // Avoid zero range
                        max_values[i] += 1e-5;
                    }
                }

                self.min_values = Some(min_values);
                self.max_values = Some(max_values);

                debug!(
                    "Initialized scalar quantizer with {} dimensions",
                    self.dimension
                );
                Ok(())
            }
            QuantizationMethod::ProductQuantization => {
                // TODO: Implement product quantization
                anyhow::bail!("Product quantization not yet implemented");
            }
        }
    }

    /// Quantize a vector according to the configured method
    pub fn quantize(&mut self, vector: &Vector, id: Option<&str>) -> Result<Vec<u8>> {
        // Check cache first if ID is provided
        if let Some(id) = id {
            if let Some(cached) = self.quantization_cache.get(id) {
                return Ok(cached.clone());
            }
        }

        // Check dimensions
        if vector.dimension() != self.dimension {
            anyhow::bail!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.dimension()
            );
        }

        let result =
            match self.method {
                QuantizationMethod::None => {
                    // Just convert f32 to bytes
                    let mut bytes = Vec::with_capacity(vector.values.len() * 4);
                    for &val in &vector.values {
                        bytes.extend_from_slice(&val.to_le_bytes());
                    }
                    bytes
                }
                QuantizationMethod::Scalar8Bit => {
                    // Ensure we've been initialized
                    let min_values = self.min_values.as_ref().ok_or_else(|| {
                        anyhow::anyhow!("Quantizer not initialized with min_values")
                    })?;
                    let max_values = self.max_values.as_ref().ok_or_else(|| {
                        anyhow::anyhow!("Quantizer not initialized with max_values")
                    })?;

                    // Convert each dimension to 8-bit
                    let mut bytes = Vec::with_capacity(vector.values.len());
                    for (i, &val) in vector.values.iter().enumerate() {
                        let range = max_values[i] - min_values[i];
                        if range <= 0.0 {
                            bytes.push(0);
                            continue;
                        }

                        let normalized = (val - min_values[i]) / range;
                        let quantized = (normalized * 255.0).round() as u8;
                        bytes.push(quantized);
                    }

                    bytes
                }
                QuantizationMethod::ProductQuantization => {
                    // TODO: Implement product quantization
                    anyhow::bail!("Product quantization not yet implemented");
                }
            };

        // Add to cache if ID is provided
        if let Some(id) = id {
            self.quantization_cache
                .insert(id.to_string(), result.clone());
        }

        Ok(result)
    }

    /// Dequantize a vector according to the configured method
    pub fn dequantize(&self, bytes: &[u8]) -> Result<Vector> {
        match self.method {
            QuantizationMethod::None => {
                // Ensure byte length is correct
                if bytes.len() != self.dimension * 4 {
                    anyhow::bail!(
                        "Byte length mismatch: expected {}, got {}",
                        self.dimension * 4,
                        bytes.len()
                    );
                }

                // Convert bytes to f32
                let mut values = Vec::with_capacity(self.dimension);
                for i in 0..self.dimension {
                    let offset = i * 4;
                    let mut val_bytes = [0u8; 4];
                    val_bytes.copy_from_slice(&bytes[offset..offset + 4]);
                    values.push(f32::from_le_bytes(val_bytes));
                }

                Ok(Vector::new(values))
            }
            QuantizationMethod::Scalar8Bit => {
                // Ensure byte length is correct
                if bytes.len() != self.dimension {
                    anyhow::bail!(
                        "Byte length mismatch: expected {}, got {}",
                        self.dimension,
                        bytes.len()
                    );
                }

                // Ensure we've been initialized
                let min_values = self
                    .min_values
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Quantizer not initialized with min_values"))?;
                let max_values = self
                    .max_values
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Quantizer not initialized with max_values"))?;

                // Convert each byte to f32
                let mut values = Vec::with_capacity(self.dimension);
                for (i, &byte) in bytes.iter().enumerate() {
                    let range = max_values[i] - min_values[i];
                    let normalized = byte as f32 / 255.0;
                    let val = normalized * range + min_values[i];
                    values.push(val);
                }

                Ok(Vector::new(values))
            }
            QuantizationMethod::ProductQuantization => {
                // TODO: Implement product quantization
                anyhow::bail!("Product quantization not yet implemented");
            }
        }
    }

    /// Get the byte size of a quantized vector
    pub fn quantized_size(&self) -> usize {
        match self.method {
            QuantizationMethod::None => self.dimension * 4, // 4 bytes per f32
            QuantizationMethod::Scalar8Bit => self.dimension, // 1 byte per dimension
            QuantizationMethod::ProductQuantization => {
                // TODO: Implement product quantization
                0
            }
        }
    }

    /// Get the quantization method
    pub fn method(&self) -> QuantizationMethod {
        self.method
    }

    /// Clear the quantization cache
    pub fn clear_cache(&mut self) {
        self.quantization_cache.clear();
    }

    /// Get the dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Get quantization parameters as JSON for serialization
    pub fn parameters_json(&self) -> serde_json::Value {
        match self.method {
            QuantizationMethod::None => {
                serde_json::json!({
                    "method": "none",
                    "dimension": self.dimension,
                })
            }
            QuantizationMethod::Scalar8Bit => {
                serde_json::json!({
                    "method": "scalar_8bit",
                    "dimension": self.dimension,
                    "min_values": self.min_values,
                    "max_values": self.max_values,
                })
            }
            QuantizationMethod::ProductQuantization => {
                serde_json::json!({
                    "method": "product_quantization",
                    "dimension": self.dimension,
                    // TODO: Add product quantization parameters
                })
            }
        }
    }

    /// Create from JSON parameters
    pub fn from_parameters_json(json: &serde_json::Value) -> Result<Self> {
        let method_str = json["method"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'method' field in quantization parameters"))?;

        let dimension = json["dimension"].as_u64().ok_or_else(|| {
            anyhow::anyhow!("Missing 'dimension' field in quantization parameters")
        })? as usize;

        let method = match method_str {
            "none" => QuantizationMethod::None,
            "scalar_8bit" => QuantizationMethod::Scalar8Bit,
            "product_quantization" => QuantizationMethod::ProductQuantization,
            _ => anyhow::bail!("Unknown quantization method: {}", method_str),
        };

        let mut quantizer = Self::new(method, dimension);

        // Load parameters for scalar quantization
        if method == QuantizationMethod::Scalar8Bit {
            if let (Some(min_arr), Some(max_arr)) =
                (json["min_values"].as_array(), json["max_values"].as_array())
            {
                let min_values = min_arr
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect();
                let max_values = max_arr
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect();

                quantizer.min_values = Some(min_values);
                quantizer.max_values = Some(max_values);
            } else {
                anyhow::bail!("Missing min_values or max_values for scalar quantization");
            }
        }

        Ok(quantizer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectordb::types::Vector;

    #[test]
    fn test_scalar_quantization() -> Result<()> {
        // Create vectors
        let vectors = vec![
            Vector::new(vec![1.0, 2.0, 3.0]),
            Vector::new(vec![4.0, 5.0, 6.0]),
            Vector::new(vec![7.0, 8.0, 9.0]),
        ];

        // Create quantizer
        let mut quantizer = VectorQuantizer::new(QuantizationMethod::Scalar8Bit, 3);

        // Initialize
        quantizer.initialize(&vectors)?;

        // Quantize and dequantize
        let vector = Vector::new(vec![2.5, 3.5, 4.5]);
        let bytes = quantizer.quantize(&vector, Some("test"))?;

        // Check size
        assert_eq!(bytes.len(), 3); // 3 dimensions, 1 byte each

        // Dequantize
        let dequantized = quantizer.dequantize(&bytes)?;

        // Check results - expect some loss due to quantization
        for (i, &val) in vector.values.iter().enumerate() {
            let diff = (val - dequantized.values[i]).abs();
            assert!(
                diff < 0.1,
                "Dimension {}: {} vs {}",
                i,
                val,
                dequantized.values[i]
            );
        }

        // Test cache
        let bytes2 = quantizer.quantize(&vector, Some("test"))?;
        assert_eq!(bytes, bytes2); // Should be identical (from cache)

        Ok(())
    }

    #[test]
    fn test_no_quantization() -> Result<()> {
        // Create vectors
        let vector = Vector::new(vec![1.0, 2.0, 3.0]);

        // Create quantizer
        let mut quantizer = VectorQuantizer::new(QuantizationMethod::None, 3);

        // No initialization needed

        // Quantize
        let bytes = quantizer.quantize(&vector, None)?;

        // Check size
        assert_eq!(bytes.len(), 12); // 3 dimensions, 4 bytes each

        // Dequantize
        let dequantized = quantizer.dequantize(&bytes)?;

        // Check results - should be exact
        assert_eq!(vector.values, dequantized.values);

        Ok(())
    }

    #[test]
    fn test_json_serialization() -> Result<()> {
        // Create vectors
        let vectors = vec![
            Vector::new(vec![1.0, 2.0, 3.0]),
            Vector::new(vec![4.0, 5.0, 6.0]),
        ];

        // Create quantizer
        let mut quantizer = VectorQuantizer::new(QuantizationMethod::Scalar8Bit, 3);

        // Initialize
        quantizer.initialize(&vectors)?;

        // Get parameters as JSON
        let json = quantizer.parameters_json();

        // Create new quantizer from JSON
        let quantizer2 = VectorQuantizer::from_parameters_json(&json)?;

        // Check that they're the same
        assert_eq!(quantizer.method, quantizer2.method);
        assert_eq!(quantizer.dimension, quantizer2.dimension);

        // Check min/max values
        if let (Some(min1), Some(min2)) = (&quantizer.min_values, &quantizer2.min_values) {
            assert_eq!(min1, min2);
        } else {
            panic!("Min values missing");
        }

        if let (Some(max1), Some(max2)) = (&quantizer.max_values, &quantizer2.max_values) {
            assert_eq!(max1, max2);
        } else {
            panic!("Max values missing");
        }

        Ok(())
    }
}

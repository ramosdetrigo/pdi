use crate::pixel_array::PixelArray;
use rayon::prelude::*;

pub trait Transform {
    fn with<OP>(&self, op: OP) -> Self
    where
        OP: Fn(&mut f64) + Sync + Send;

    fn threshold(&self, threshold: f64) -> Self;

    fn neg(&self) -> Self;

    fn brightness(&self, b: f64) -> Self;

    fn gamma(&self, c: f64, y: f64) -> Self;

    fn hide(&self, msg: &str) -> Self;

    fn read(&self) -> String;

    // TODO: linear por partes
}

pub trait TransformMut {
    fn apply<OP>(&mut self, op: OP)
    where
        OP: Fn(&mut f64) + Sync + Send;

    fn threshold_mut(&mut self, threshold: f64);

    fn neg_mut(&mut self);

    fn brightness_mut(&mut self, b: f64);

    fn gamma_mut(&mut self, c: f64, y: f64);

    fn hide_mut(&mut self, msg: &str);

    // TODO: linear por partes

    // TODO: esteganografia
}

impl Transform for PixelArray {
    fn with<OP>(&self, op: OP) -> Self
    where
        OP: Fn(&mut f64) + Sync + Send,
    {
        let mut new = self.clone();
        new.apply(op);
        new
    }

    fn brightness(&self, c: f64) -> Self {
        let mut new = self.clone();
        new.brightness_mut(c);
        new
    }

    fn gamma(&self, c: f64, y: f64) -> Self {
        let mut new = self.clone();
        new.gamma_mut(c, y);
        new
    }

    fn neg(&self) -> Self {
        let mut new = self.clone();
        new.neg_mut();
        new
    }

    fn threshold(&self, threshold: f64) -> Self {
        let mut new = self.clone();
        new.threshold_mut(threshold);
        new
    }

    fn hide(&self, msg: &str) -> Self {
        let mut new = self.clone();
        new.hide_mut(msg);
        new
    }

    fn read(&self) -> String {
        let mut chars: Vec<u8> = Vec::new();

        for pixels in self.data().chunks_exact(8) {
            let mut byte = 0u8;
            for i in 0..8 {
                let pixel = (pixels[i].clamp(0.0, 1.0) * 255.0) as u8;
                let bit = pixel & 1;
                byte |= bit << i;
            }
            if byte == 0 {
                break;
            }
            chars.push(byte);
        }

        String::from_utf8_lossy(&chars).into_owned()
    }
}

impl TransformMut for PixelArray {
    fn apply<OP>(&mut self, op: OP)
    where
        OP: Fn(&mut f64) + Sync + Send,
    {
        self.data_mut().par_iter_mut().for_each(op);
    }

    fn brightness_mut(&mut self, c: f64) {
        self.apply(|p| *p *= c);
    }

    fn gamma_mut(&mut self, c: f64, y: f64) {
        self.apply(|p| *p = p.powf(y) * c);
    }

    fn neg_mut(&mut self) {
        self.apply(|p| *p = 1.0 - *p);
    }

    fn threshold_mut(&mut self, threshold: f64) {
        self.apply(|p| *p = if *p > threshold { 1.0 } else { 0.0 });
    }

    fn hide_mut(&mut self, msg: &str) {
        if !msg.is_ascii() {
            panic!("Can only encode ascii characters!");
        };

        // Joga cada bit do byte no LSB
        let mut bytes = msg.to_string().into_bytes();
        bytes.push(0); // null-terminated string
        let bits: Vec<u8> = bytes
            .into_par_iter()
            .flat_map(|b| (0..8).map(|i| (b & (1 << i)) >> i).collect::<Vec<u8>>())
            .collect();

        // Esconde os bits na imagem pixel por pixel
        for i in 0..bits.len() {
            if i > self.len() {
                // para de esconder se passou do tamanho da imagem
                break;
            }
            let mut b = (self[i].clamp(0.0, 1.0) * 255.0) as u8;
            b = (b & 0b11111110) + bits[i]; // apaga o último bit e sobrescreve
            self[i] = b as f64 / 255.0
        }
    }
}

use super::checksumpair::ChecksumPair;

#[derive(Debug)]
pub struct ChainingHash {
    hash_array: Vec<Vec<ChecksumPair>>,
    array_size: i32,
    index: i32,
}

impl ChainingHash {
    pub fn new(size: i32) -> Self {
        let mut array: Vec<Vec<ChecksumPair>> = Vec::new();
        for _ in 0..size {
            array.push(Vec::new());
        }

        ChainingHash {
            array_size: size,
            hash_array: array,
            index: 0,
        }
    }

    pub fn hash_function(&self, p_key: &ChecksumPair) -> i32 {
        p_key.hash_code() % self.array_size
    }

    pub fn insert(&mut self, p_key: &ChecksumPair) {
        let hash_value = self.hash_function(p_key);
        self.hash_array[hash_value as usize].push(p_key.clone());
    }

    pub fn delete(&mut self, p_key: &ChecksumPair) {
        let hash_value = self.hash_function(p_key);

        let array = &mut self.hash_array[hash_value as usize];
        if let Some(pos) = array.iter().position(|pk| pk == p_key) {
            array.remove(pos);
        }
    }

    pub fn find(&mut self, p_key: &ChecksumPair) -> Option<ChecksumPair> {
        let hash_value = self.hash_function(p_key);

        let array = &self.hash_array[hash_value as usize];

        for i in 0..array.len() {
            let pair = &array[i];
            if pair.weak == p_key.weak {
                self.index = i as i32;
                return Some(pair.clone());
            }
        }
        None
    }

    pub fn find_match(&mut self, p_key: &ChecksumPair) -> Option<ChecksumPair> {
        let hash_value = self.hash_function(p_key);

        let array = &self.hash_array[hash_value as usize];
        if array.is_empty() {
            return None;
        }
        let pair = &array[self.index as usize];

        if pair.weak == p_key.weak && pair.strong == p_key.strong {
            return Some(pair.clone());
        }

        for i in 0..array.len() {
            let pair = &array[i as usize];
            if pair.weak == p_key.weak && pair.strong == p_key.strong {
                return Some(pair.clone());
            }
        }

        None
    }
}

impl Clone for ChainingHash {
    fn clone(&self) -> Self {
        Self {
            hash_array: self.hash_array.clone(),
            array_size: self.array_size,
            index: self.index,
        }
    }
}

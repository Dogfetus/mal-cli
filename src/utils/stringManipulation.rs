use std::fmt::Display;


pub struct DisplayString{
    text: Vec<String>, 
}

#[allow(dead_code)]
impl DisplayString{
    pub fn new() -> Self{
        DisplayString {
            text: Vec::new(),
        }
    }

    pub fn add<T: Display>(mut self, text: T) -> Self{
        self.text.push(text.to_string());
        self
    }

    pub fn uppercase(mut self, char_index: usize) -> Self {
        let mut current_pos = 0;
        
        for word in self.text.iter_mut() {
            let word_len = word.chars().count();
            
            if char_index < current_pos + word_len {
                let pos_in_word = char_index - current_pos;
                let mut chars: Vec<char> = word.chars().collect();
                
                if pos_in_word < chars.len() {
                    chars[pos_in_word] = chars[pos_in_word].to_uppercase().next().unwrap_or(chars[pos_in_word]);
                    *word = chars.into_iter().collect();
                }
                break;
            }
            
            current_pos += word_len;
        }
        
        self
    }

    pub fn capitalize(mut self, word_index: usize) -> Self {
        if let Some(word) = self.text.get_mut(word_index) {
            if let Some(first_char) = word.chars().next() {
                let uppercase_first = first_char.to_uppercase().collect::<String>();
                let rest = &word[first_char.len_utf8()..];
                *word = format!("{}{}", uppercase_first, rest);
            }
        }
        self
    }
    
    pub fn build(self, format_str: &str) -> String {
        let mut result = format_str.to_string();
        
        for (i, value) in self.text.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            result = result.replace(&placeholder, value);
        }
        
        result
    }
}


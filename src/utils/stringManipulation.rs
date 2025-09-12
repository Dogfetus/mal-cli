use chrono::{DateTime, NaiveDate};
use std::fmt::Display;

pub struct DisplayString {
    text: Vec<String>,
}

#[allow(dead_code)]
impl DisplayString {
    pub fn new() -> Self {
        DisplayString { text: Vec::new() }
    }

    pub fn add<T: Display>(mut self, text: T) -> Self {
        self.text.push(text.to_string());
        self
    }

    // make a character uppercase at a specific index
    pub fn uppercase(mut self, char_index: usize) -> Self {
        let mut current_pos = 0;

        for word in self.text.iter_mut() {
            let word_len = word.chars().count();

            if char_index < current_pos + word_len {
                let pos_in_word = char_index - current_pos;
                let mut chars: Vec<char> = word.chars().collect();

                if pos_in_word < chars.len() {
                    chars[pos_in_word] = chars[pos_in_word]
                        .to_uppercase()
                        .next()
                        .unwrap_or(chars[pos_in_word]);
                    *word = chars.into_iter().collect();
                }
                break;
            }
            current_pos += word_len;
        }
        self
    }

    // make a specific word uppercase the first letter
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

pub fn format_date(date_str: &str) -> String {
    // Try parsing as RFC3339 (2025-07-06T15:08:00Z)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return dt.format("%B %d, %Y at %I:%M %p").to_string();
    }
    // Try parsing date with time but no timezone (2025-07-06T15:08)
    if let Ok(dt) = DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M") {
        return dt.format("%B %d, %Y at %I:%M %p").to_string();
    }
    // Try parsing just date (2025-07-06) - This is the key fix
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return date.format("%B %d, %Y").to_string();
    }
    // Try parsing other date formats
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
        return date.format("%B %d, %Y").to_string();
    }
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%m/%d/%Y") {
        return date.format("%B %d, %Y").to_string();
    }
    date_str.to_string()
}

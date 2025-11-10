use std::collections::HashMap as StdHashMap;
use fancy_regex::Regex;
use pyo3::prelude::*;

// Default GPT-4 style regex pattern for splitting text
const GPT4_PATTERN: &str = r"'(?i:[sdmt]|ll|ve|re)|[^\r\n\p{L}\p{N}]?+\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]++[\r\n]*|\s*[\r\n]|\s+(?!\S)|\s+";

type TokenId = u32;
type Pair = (TokenId, TokenId);

#[pyclass]
pub struct Tokenizer {
    pub merges: StdHashMap<Pair, TokenId>,
    pub pattern: String,
    compiled_pattern: Regex,
}

impl Tokenizer {
    fn merge_pair(&self, tokens: &[TokenId], pair: Pair, new_id: TokenId) -> Vec<TokenId> {
        let mut result = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            if i + 1 < tokens.len() && tokens[i] == pair.0 && tokens[i+1] == pair.1 {
                result.push(new_id);
                i += 2;
            } else {
                result.push(tokens[i]);
                i += 1;
            }
        }
        result
    }

    fn find_lowest_token_merge_in_chunk(&self, chunk_tokens: &[TokenId]) -> Option<(Pair, TokenId)> {
        let mut best: Option<(Pair, TokenId)> = None;

        for window in chunk_tokens.windows(2) {
            let pair = (window[0], window[1]);
            if let Some(&token_id) = self.merges.get(&pair) {
                if best.is_none() || token_id < best.unwrap().1 {
                    best = Some((pair, token_id));
                }
            }
        }

        best
    }
}

#[pymethods]
impl Tokenizer {
    #[new]
    pub fn new() -> Self {
        Self {
            merges: StdHashMap::new(),
            pattern: GPT4_PATTERN.to_string(),
            compiled_pattern: Regex::new(GPT4_PATTERN).expect("GPT4 pattern should be valid"),
        }
    }

    pub fn train_from_iterator(
        &mut self,
        py: Python<'_>,
        iterator: &Bound<'_, PyAny>,
        vocab_size: u32,
    ) -> PyResult<()> {
        // Collect all strings from the iterator
        let mut all_text = String::new();

        for item in iterator.iter()? {
            let text: String = item?.extract()?;
            all_text.push_str(&text);
        }

        // Call existing train() method
        self.train(&all_text, vocab_size);

        Ok(())
    }

    pub fn train(&mut self, text: &str, vocab_size: u32) {

        let mut chunk_tokens_list: Vec<Vec<TokenId>> = Vec::new();

        for match_result in self.compiled_pattern.find_iter(text) {
            let chunk = match_result.expect("regex match failed").as_str();
            let tokens: Vec<TokenId> = chunk.bytes().map(|b| b as u32).collect();
            chunk_tokens_list.push(tokens);
        }

        let mut current_token = 256;
        while current_token < vocab_size {
            let mut token_count: StdHashMap<Pair, u32> = StdHashMap::new();
            // count pairs
            for chunk_tokens in &chunk_tokens_list {
                for pair in chunk_tokens.windows(2) {
                    let token_pair: Pair = (pair[0], pair[1]);
                    *token_count.entry(token_pair).or_insert(0) += 1;
                }
            }
            // get max pair and replace
            if let Some((max_pair, _count)) = token_count.iter().max_by_key(|&(_,count)| count) {
                self.merges.insert(*max_pair, current_token);

                for i in 0..chunk_tokens_list.len() {
                    chunk_tokens_list[i] = self.merge_pair(&chunk_tokens_list[i], *max_pair, current_token);
                }
                current_token += 1;
            } else {
                // no more pairs
                break;
            }
        }
    }

    pub fn encode(&self, text: &str) -> Vec<TokenId> {
        let mut tokens = Vec::new();

        let mut chunk_tokens_list: Vec<Vec<TokenId>> = Vec::new();

        for match_result in self.compiled_pattern.find_iter(text) {
            let chunk = match_result.expect("regex match failed").as_str();
            let chunk_tokens: Vec<TokenId> = chunk.bytes().map(|b| b as u32).collect();
            chunk_tokens_list.push(chunk_tokens);
        }

        for i in 0..chunk_tokens_list.len() {
            loop {
                let pair_token_id_opt = self.find_lowest_token_merge_in_chunk(&chunk_tokens_list[i]);
                if let Some((pair, token_id)) = pair_token_id_opt {
                    chunk_tokens_list[i] = self.merge_pair(&chunk_tokens_list[i], pair, token_id);
                } else {
                    break;
                }
            }
        }

        for chunk_tokens in chunk_tokens_list {
            tokens.extend(chunk_tokens);
        }

        tokens 
    }

    pub fn get_pattern(&self) -> String {
        self.pattern.clone()
    }

    pub fn get_mergeable_ranks(&self) -> Vec<(Vec<u8>, u32)> {
        let mut mergeable_ranks = Vec::new();

        for i in 0..256 {
            mergeable_ranks.push((vec![i as u8], i));
        }

        let mut sorted_merges: Vec<_> = self.merges.iter().collect();
        sorted_merges.sort_by_key(|&(_, &token_id)| token_id);

        for (&(left, right), &token_id) in sorted_merges {
            let mut merged_bytes = mergeable_ranks[left as usize].0.clone();
            merged_bytes.extend(&mergeable_ranks[right as usize].0);
            mergeable_ranks.push((merged_bytes, token_id));
        }

        mergeable_ranks
    }
    
}

#[pymodule]
fn mybpe(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tokenizer>()?;
    Ok(())
}

//! Quick manual check of the real extraction pipeline without the GUI.
//!
//!   cargo run -p nlp --example demo            # built-in Traditional sample
//!   cargo run -p nlp --example demo -- <file>  # a real .md from your vault
//!
//! Prints the ranked annotations so you can eyeball jieba's output quality
//! (especially Traditional Chinese, which uses the HMM fallback).

use std::env;
use std::fs;

const SAMPLE: &str = "# 會議記錄\n\n\
今天和愛因斯坦討論了相對論。相對論是現代物理學的基礎。\n\
我們也談到量子力學，量子力學在半導體產業很重要。\n\
台北是台灣的首都，台北的天氣今天很好。";

fn main() {
    let text = match env::args().nth(1) {
        Some(p) => match fs::read_to_string(&p) {
            Ok(t) => {
                println!("# source: {p}\n");
                t
            }
            Err(e) => {
                eprintln!("read {p}: {e}");
                std::process::exit(1);
            }
        },
        None => {
            println!("# source: built-in Traditional-Chinese sample\n");
            SAMPLE.to_string()
        }
    };

    let anns = nlp::extract_nouns(&text);
    println!("extracted {} unique terms (top 40):\n", anns.len());
    println!("{:>5}  {:<6}  term", "count", "pos");
    println!("{}", "-".repeat(28));
    for a in anns.iter().take(40) {
        println!("{:>5}  {:<6}  {}", a.count, a.pos, a.text);
    }
}

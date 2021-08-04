// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);
    let mut mut_secret_word_chars: Vec<char> = secret_word.chars().collect();
    let mut guess_result: Vec<char> = secret_word.as_str().chars().collect::<Vec<char>>().iter().map(|_| '-').collect();
    let mut remain_chances = NUM_INCORRECT_GUESSES;
    let mut guess = String::new();
    let mut current = String::new();
    loop {
        println!("The word so far is {}", guess_result.iter().collect::<String>());
        println!("You have {} guesses left", remain_chances);
        println!("You have guessed the following letters: {}", current);
        if remain_chances == 0 {
            println!("You have wasted all the chances, the answer is {}", secret_word);
            break;
        }
        if secret_word.len() == current.len() {
            println!("Conguratulations! You find the correct answer!");
            break;
        }
        io::stdin()
        .read_line(&mut guess)
        .expect("Failed to read line");
        let letters: Vec<char>= guess.trim().chars().collect();
        guess.clear();
        io::stdout().flush();
        match mut_secret_word_chars.iter().position(|&x| x == letters[0]) {
            Some(value) => { 
                guess_result[value] = secret_word_chars[value];
                current.push(letters[0]);  
                mut_secret_word_chars[value] = '#';
            }
            None => { 
                println!("You may forget the letter! There is no {} in the origin word", letters[0]);
                remain_chances -= 1;
            }
        }
    }
}

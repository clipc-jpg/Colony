


use std::process::{Command, Child, ChildStdout};
use std::sync::{Arc, Mutex};
use std::io::{Read, BufReader, BufRead}; // chunk.bytes()

use duct::Handle as ProcessHandle;
use itertools::Itertools; // Iterator.collect_vec();









//################################################################################
//## Std library Child Processes
//################################################################################

pub async fn bufread_child_stdout_into_messages(
    output_collection: &mut Arc<Mutex<Vec<String>>>,
    process_stdout: ChildStdout) {

    // Stream output.
    let lines = BufReader::new(process_stdout).lines();
    for line in lines {
        let mut n = 0;
        while n<3 {
            if let Err(ref _unusable) = line {continue;}
            match output_collection.lock() {
                Ok(ref mut mutex) => {
                    let ln = line.unwrap();
                    mutex.push(ln);
                    break;
                },
                Err(_) => {n += 1;}
            }
        }
    };
}



pub fn bufread_child_stdout_bytes_into_messages(
    output_collection: &mut Arc<Mutex<Vec<String>>>,
    process: &mut Child) {
    let mut stdout = process.stdout.take().unwrap();

    //bufread_stdout_bytes_into_messages(output_collection, &mut stdout);
    bufread_stdout_bytes_into_messages_v2(output_collection, &mut stdout);
}

pub fn bufread_stdout_bytes_into_messages(
    output_collection: &mut Arc<Mutex<Vec<String>>>,
    stdout: &mut ChildStdout) {

    let mut accumulated_buf = Vec::new() as Vec<u8>;
    let mut reserve_buf = Vec::new() as Vec<u8>;
    let mut current_line = String::new();
    let mut current_line_revised = Vec::new() as Vec<char>;

    for byte in stdout.bytes() {
        let byte = match byte {
            Ok(b) => b,
            Err(_) => continue
        };
        accumulated_buf.push(byte);

        // all chunks contain valid utf8 and invalid utf8
        // all but the last invalid parts will be converted to the invalid character "\u{FFFD}"
        // all valid chunks need to split by the Carriage return character, in order to backtrack inputs as e.g. progrss bars do

        let mut chunks = accumulated_buf.utf8_chunks().collect_vec();
        let last_chunk = chunks.pop();

        for chunk in &chunks {
            current_line.push_str(chunk.valid());
            if !chunk.invalid().is_empty() { current_line.push_str("\u{FFFD}"); }
        }
        chunks.clear();
        match last_chunk {
            Some(chunk) => {
                current_line.push_str(chunk.valid());
                for byte in chunk.invalid().bytes() {
                    match byte {
                        Ok(b) => { reserve_buf.push(b); },
                        Err(_) => continue
                    }
                }
            },
            None => {}
        }

        std::mem::swap(&mut accumulated_buf, &mut reserve_buf); reserve_buf.clear();

        // mutating output collection
        // the following code section is actually wrong, since backspace usually deletes a codepoint (==char)
        // but sometimes, grapheme clusters (or parts of them) are considered inseparable and deleted as a whole
        // Todo: find source that explains most correct approach (e.g. approach of browsers or text editors)
        current_line_revised.clear();
        for ch in current_line.chars() {
            if ch == char::from_u32(8).expect("Char 8 should have converted to BACKSPACE") { // Backspace
                // if no chars is left, then the previous lines remain untouched
                // cannot dsiplay multiline outputs (e.g. TUIs)
                current_line_revised.pop();
            } else if ch == char::from_u32(13).expect("Char 13 should have converted to CARRIAGE RETURN") {  // Carriage Return: do nothing
            } else if ch == char::from_u32(10).expect("Char 10 should have converted to LINE FEED") { // Line Feed
                match output_collection.lock() {
                    Ok(mut lines) => {
                        lines.push(current_line_revised.iter().collect());
                        current_line_revised.clear();
                    },
                    Err(_) => {break;}
                }
            } else if false && current_line_revised.len() > 1000 {
                match output_collection.lock() {
                    Ok(mut lines) => {
                        lines.push(current_line_revised.iter().collect());
                        current_line_revised.clear();
                    },
                    Err(_) => {break;}
                }
            } else {
                current_line_revised.push(ch);
            }
        }

        current_line = current_line_revised.iter().collect();

        match output_collection.lock() {
            Ok(mut lines) => {
                if lines.len() == 0 {
                    lines.push(current_line.clone());
                } else {
                    match lines.last_mut() {
                        Some(last_line) => {
                            current_line.clone_into(last_line);
                        },
                        None => {} // impossible
                    }
                }
            },
            Err(_) => {break;} // lock is poisened and will never return again
        }
    }
}


pub fn bufread_stdout_bytes_into_messages_v2(
    output_collection: &mut Arc<Mutex<Vec<String>>>,
    stdout: &mut ChildStdout) {

    println!("##### Using new 'bufread_stdout_bytes_into_messages' implementation");


    // typical screen size: 30 rows, 120 columns => 50 screens ≈ 180000 characters
    let rows = 30*2; // keep two screens of state ready
    let cols = 120;
    let scrollback = 0;
    let mut term_parser = vt100::Parser::new(rows, cols, scrollback);


    let mut buf = [0 as u8; 4096];
    let mut bufreader = BufReader::new(stdout);

    while let Ok(n) = bufreader.read(&mut buf[..]) {

        if n == 0 { break; } // stdout has closed

        term_parser.process(&buf[..n]);
        match output_collection.lock() {
            Ok(mut lines) => {
                match lines.last().is_none() {
                    // only need to append
                    true => {
                        println!("##### Appending content to empty lines");
                        let cont = term_parser.screen().rows(0, cols);
                        for l in cont { lines.push(l.to_owned()); }
                    },
                    // needs "parsing", line substitutions or appends
                    // last line is always > 0
                    false => {

                        println!("Lines: {lines:?}");

                        let last_line = lines.len();

                        println!("##### length of 'lines': {last_line}");

                        // take care not to underflow, overflow from 2*u16 not possible
                        let no_revisit = last_line-(rows as usize).min(last_line); // at most revisit two terminal screens == rows
                        let parser_contents = term_parser.screen().contents();
                        let mut new_lines = parser_contents.lines();
                        let first_new_line = new_lines.next();

                        // empty strings produce a empty Lines iterators
                        if first_new_line.is_some() {
                            let mut first_new_line = first_new_line.unwrap();

                            // first_new_line should appear in lines
                            // if yes, replace all lines after that
                            // else, simply append all new lines
                            let newest_line = lines
                            .iter()
                            .enumerate()
                            .skip(no_revisit)
                            .find(|(_k, l)| **l == first_new_line) // compare contents, not pointers
                            .map(|(k, _l)| k)
                            .unwrap_or(last_line + 1);

                            if newest_line > last_line {
                                lines.push(first_new_line.to_owned());
                                for l in new_lines { lines.push(l.to_owned()); }
                            } else {
                                lines.truncate(newest_line);
                                for l in new_lines { lines.push(l.to_owned()); }
                            }
                        } else {
                            println!("##### No content to add to 'lines'");
                        }

                    }
                }
            },
            Err(_) => {break;} // lock is poisened and will never return again
        }
    }
    println!("##### Ended reading from process");
    //TODO: Update lines after stdout has been closed, too.
}




//################################################################################
//## Duct Library Processes
//################################################################################







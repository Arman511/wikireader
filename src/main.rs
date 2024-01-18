use confy;
use random_word::Lang;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SessionConfig {
    pub article_name: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            article_name: String::from("Cheese"),
        }
    }
}

impl Clone for SessionConfig {
    fn clone(&self) -> Self {
        Self {
            article_name: self.article_name.clone(),
        }
    }
}

fn get_options(msg: String, options: Vec<String>) -> usize {
    loop {
        clear();
        println!("{}", msg);
        options.iter().enumerate().for_each(|(i, option)| {
            println!("{}: {}", i + 1, option);
        });
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim().parse::<usize>() {
            Ok(num) => {
                if num > 0 && num <= options.len() {
                    return num;
                } else {
                    println!("Please enter a valid number - press enter to continue");
                    let _ = std::io::stdin().read_line(&mut String::new());
                    continue;
                }
            }
            Err(_) => {
                println!("Please enter a valid number - press enter to continue");
                let _ = std::io::stdin().read_line(&mut String::new());
                continue;
            }
        };
    }
}

fn main() {
    let mut article: Vec<String>;
    loop {
        let option = menu();
        match option {
            1 => article = get_previous_article(),
            2 => article = search_for_article(),
            3 => article = get_random_article(),
            4 => break,
            _ => continue,
        }
        clear();
        show_article(article);
    }
}

fn show_article(article: Vec<String>) {
    for line in article {
        println!("{}", line);
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}

fn get_previous_article() -> Vec<String> {
    let article = confy::load("wikireader", "SessionConfig").unwrap_or_default();
    get_article(article)
}

fn search_for_article() -> Vec<String> {
    println!("Search for article");
    let mut article_name;
    loop {
        let mut input = String::new();
        println!("Enter article name: ");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        article_name = input.trim().to_owned().replace(" ", "_");
        let not_article_is_disambiguation = check_article(article_name.to_owned());
        if not_article_is_disambiguation {
            break;
        }
    }
    get_article(article_name)
}

fn get_random_article() -> Vec<String> {
    let mut word;
    loop {
        word = random_word::gen(Lang::En);
        let not_article_is_disambiguation = check_article(word.to_owned());
        if not_article_is_disambiguation {
            break;
        }
    }

    get_article(word.to_string())
}

fn check_article(article_name: String) -> bool {
    let response =
        reqwest::blocking::get(&format!("https://en.wikipedia.org/wiki/{}", article_name));

    match response {
        Ok(response) => return response.text().unwrap().contains(" may refer to:"),
        Err(_) => return false,
    }
}

fn get_article(article_name: String) -> Vec<String> {
    todo!()
}

fn menu() -> usize {
    get_options(
        "Test".to_owned(),
        vec![
            String::from("Read previous article"),
            String::from("Search for article"),
            String::from("Get random article"),
            String::from("Exit"),
        ],
    )
}

fn clear() {
    print!("{}[2J", 27 as char);
}

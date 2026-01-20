//! Example: Lexing an NGINX configuration

use nginx_discovery::parser::Lexer;

fn main() {
    let config = r#"
# NGINX Configuration
server {
    listen 80;
    server_name example.com;

    location / {
        root /var/www/html;
        index index.html;
    }

    access_log /var/log/nginx/access.log combined;
}
"#;

    println!("Input:\n{}\n", config);
    println!("Tokens:\n");

    let mut lexer = Lexer::new(config);

    match lexer.tokenize() {
        Ok(tokens) => {
            for (i, token) in tokens.iter().enumerate() {
                if token.kind == nginx_discovery::parser::TokenKind::Eof {
                    break;
                }
                println!("{:3}: {:?} at line {}, col {}",
                    i,
                    token.kind,
                    token.span.line,
                    token.span.col
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e.detailed());
        }
    }
}
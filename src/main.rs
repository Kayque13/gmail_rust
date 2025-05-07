use colored::Colorize;
use mailparse::parse_mail;
use native_tls::TlsConnector;
use regex::Regex;
use std::net::TcpStream;
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let imap_server = "imap.gmail.com";
    let imap_port = 993;
    let username = "kayque.cafe13@gmail.com";
    let password = "rkcz tzcc jawn vivv";

    let tls = TlsConnector::builder().build()?;
    let tcp_stream = TcpStream::connect((imap_server, imap_port))?;
    let tls_stream = tls.connect(imap_server, tcp_stream)?;

    let client = imap::Client::new(tls_stream);
    let mut imap_session = client
        .login(username, password)
        .map_err(|e| format!("Erro de login: {}", e.0))?;

    imap_session.select("INBOX")?;

    let messages = imap_session.fetch("1:50", "RFC822")?;

    println!("{}", "===Lista de Emails===".bold().cyan());
    for (i, message) in messages.iter().enumerate() {
        if let Some(body) = message.body() {
            let parsed = parse_mail(body)?;

            let from = parsed
                .headers
                .iter()
                .find(|h| h.get_key().to_lowercase() == "from")
                .map(|h| h.get_value())
                .unwrap_or("Desconhecido".to_string());

            let subject = parsed
                .headers
                .iter()
                .find(|h| h.get_key().to_lowercase() == "subject")
                .map(|h| h.get_value())
                .unwrap_or("Sem Assunto".to_string());

            let date = parsed
                .headers
                .iter()
                .find(|h| h.get_key().to_lowercase() == "date")
                .map(|h| h.get_value())
                .unwrap_or("Desconhecido".to_string());

            let body = extract_text_body(&parsed);
            let formatted_body = shorten_urls(&body);

            println!("{}", format!("---Email {} ---", i + 1).bold().yellow());
            println!("{}", format!("De: {}", from).green());
            println!("{}", format!("Assunto: {}", subject).green());
            println!("{}", format!("Data: {}", date).green());
            println!("{}", "Corpo:".bold());
            println!("{}", formatted_body);
            println!("{}", "----------------------------".yellow());
        }
    }

    imap_session.logout()?;
    println!("{}", "Sessão encerrada com sucesso".cyan());
    Ok(())
}

fn extract_text_body(parsed: &mailparse::ParsedMail) -> String {
    if parsed.ctype.mimetype == "text/plain" {
        return parsed
            .get_body()
            .unwrap_or("Erro ao decodificar corpo".to_string());
    }

    for part in &parsed.subparts {
        if part.ctype.mimetype == "text/plain" {
            return part
                .get_body()
                .unwrap_or("Erro ao decodificar corpo".to_string());
        }
    }

    "Nenhum conteúdo em texto puro encontrado".to_string()
}

fn shorten_urls(text: &str) -> String {
    let re = Regex::new(r"https?://[^\s]+").expect("Erro ao compilar regex");
    let mut result = text.to_string();

    for mat in re.find_iter(text) {
        let url_str = mat.as_str();
        if let Ok(url) = Url::parse(url_str) {
            let domain = url.domain().unwrap_or("desconhecido");
            let shortened = format!("{}/...", domain);
            result = result.replace(url_str, &shortened);
        }
    }
    result
}

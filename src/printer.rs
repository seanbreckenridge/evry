/// Handles printing logs/serializing JSON
/// Printer.print lets you specify a PrinterType to
/// filter the passed message by, allowing us
/// to print more messages to Json since the user
/// may want to parse specific parts of the logs
use serde::Serialize;

#[derive(PartialEq)]
pub enum PrinterType {
    Stderr,
    Json,
}

#[derive(Serialize)]
pub struct Message {
    /// type of message
    r#type: String,
    /// message to print
    body: String,
}

impl Message {
    pub fn new(r#type: &str, body: &str) -> Self {
        Self {
            r#type: r#type.to_string(),
            body: body.to_string(),
        }
    }

    fn intersperse(&self, delim: &str) -> String {
        format!("{}{}{}", self.r#type, delim, self.body)
    }
}

pub struct Printer {
    /// how to print these messages
    printer_type: PrinterType,
    /// messages to print
    messages: Vec<Message>,
}

impl Printer {
    pub fn new(printer_type: PrinterType) -> Self {
        Self {
            printer_type,
            messages: vec![],
        }
    }

    /// Print the message (or save it, depending on the printer_type)
    pub fn print(&mut self, message: Message, only: Option<PrinterType>) {
        let allowed = match only {
            Some(ptype) => self.printer_type == ptype,
            None => true,
        };
        if allowed {
            match self.printer_type {
                PrinterType::Stderr => eprintln!("{}", message.intersperse(":")),
                PrinterType::Json => self.messages.push(message),
            }
        }
    }

    /// shorthand for print
    /// print the given (name, body) on all PrinterTypes
    pub fn echo(&mut self, r#type: &str, body: &str) {
        self.print(Message::new(r#type, body), None)
    }

    /// serialize the messages as JSON
    fn serialize(&self) -> String {
        serde_json::to_string(&self.messages).unwrap()
    }

    /// Finalize anything before the program ends. If the printer_type
    /// was JSON, this would serialize and print all the messages
    pub fn flush(&self) {
        if self.printer_type == PrinterType::Json {
            println!("{}", self.serialize())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_serialize() {
        // create a JSON printer
        let mut p = Printer::new(PrinterType::Json);
        // print for all types
        p.print(Message::new("data dir", "~/.local/share/evry/data"), None);
        p.print(
            Message::new("tag name", "this is tag name"),
            Some(PrinterType::Json),
        );
        p.print(
            Message::new("status", "something bad happened"),
            Some(PrinterType::Json),
        );
        // shouldn't accept, since this is a Json printer
        p.print(
            Message::new("status", "this shouldnt be in the output"),
            Some(PrinterType::Stderr),
        );
        assert_eq!(p.serialize(), "[{\"type\":\"data dir\",\"body\":\"~/.local/share/evry/data\"},{\"type\":\"tag name\",\"body\":\"this is tag name\"},{\"type\":\"status\",\"body\":\"something bad happened\"}]");
    }
}

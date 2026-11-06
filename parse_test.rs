fn main() {
    let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Test\\, Escape\r\nDTSTART:20231028T143045Z\r\nEND:VEVENT\r\nEND:VCALENDAR";
    println!("Testing split...");
}

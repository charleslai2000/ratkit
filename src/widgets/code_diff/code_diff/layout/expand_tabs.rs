/// Expands tab characters to spaces using the provided tab width.
pub fn expand_tabs(input: &str, tab_width: usize) -> String {
    let tab_width = tab_width.max(1);
    let mut output = String::new();
    let mut column = 0;

    for character in input.chars() {
        if character == '\t' {
            let spaces = tab_width - (column % tab_width);
            output.push_str(&" ".repeat(spaces));
            column += spaces;
        } else {
            output.push(character);
            column += 1;
        }
    }

    output
}

use vhdl_syntax::syntax::design::DesignFile;

fn main() {
    let result = "\
library ieee;
use ieee.std_logic_1164.all;

entity baz is
end entity;

entity foo is
end entity;
    "
    .parse::<DesignFile>()
    .expect("erroneous input");
}

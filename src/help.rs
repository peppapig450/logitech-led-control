// Helper functions to print additional help messages

use crate::keyboard::colors;

pub fn print_keys_help() {
    const HELP: &str = r"Keys
----

Group List :
    logo
    indicators
    fkeys
    modifiers
    multimedia
    arrows
    numeric
    functions
    keys

Group Logo :
    logo

Group indicators :
    num_indicator, numindicator, num
    caps_indicator, capsindicator, caps
    scroll_indicator, scrollindicator, scroll
    game_mode, gamemode, game
    back_light, backlight, light

Group fkeys :
    f1 - f12

Group modifiers :
    shift_left, shiftleft, shiftl
    ctrl_left, ctrlleft, ctrll
    win_left, winleft, win_left
    alt_left, altleft, altl
    alt_right, altright, altr, altgr
    win_right, winright, winr
    menu
    ctrl_right, ctrlright, ctrlr
    shift_right, shiftright, shiftr

Group multimedia :
    mute
    play_pause, playpause, play
    stop
    previous, prev
    next

Group arrows :
    arrow_top, arrowtop, top
    arrow_left, arrowleft, left
    arrow_bottom, arrowbottom, bottom
    arrow_right, arrowright, right

Group numeric :
    num_lock, numlock
    num_slash, numslash, num/
    num_asterisk, numasterisk, num*
    num_minus, numminus, num-
    num_plus, numplus, num+
    numenter
    num0 - num9
    num_dot, numdot, num.

Group functions :
    escape, esc
    print_screen, printscreen, printscr
    scroll_lock, scrolllock
    pause_break, pausebreak
    insert, ins
    home
    page_up, pageup
    delete, del
    end
    page_down, pagedown

Group keys :
    0 - 9
    a - z
    tab
    caps_lock, capslock
    space
    backspace, back
    enter
    tilde
    minus
    equal
    open_bracket
    close_bracket
    backslash
    semicolon
    dollar
    quote
    intl_backslash
    comma
    period
    slash
";

    println!("{HELP}");
}

pub fn print_effects_help() {
    println!("Effects\n-------\n");
    println!("  -fx <effect> <target> [args]\n");
    println!("Examples:");
    println!("  -fx color keys 00ff00");
    println!("  -fx breathing logo 00ff00 0a");
    println!("  -fx cycle all 0a");
}

pub fn print_samples_help() {
    println!("Samples\n-------\n");
    println!("logi-led -p profile.txt\n    Load a profile from a file");
    println!("logi-led -k logo ff0000\n    Set the logo key red");
    println!("logi-led -a 00ff00\n    Set all keys green");
}

pub fn print_colors_help() {
    println!("Colors\n------");
    for name in colors::color_names() {
        print!("{name}");
    }
}

// If additional  strings of this nature are added they should be moved to their own module
pub const COLOR_HELP: &str = colors::COLOR_HELP;

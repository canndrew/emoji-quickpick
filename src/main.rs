use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::thread;
use std::str::FromStr;
use std::ffi::CString;
use std::ptr;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, WindowPosition, SearchEntry, ListBox, Orientation, Label, Widget, ListBoxRow, Inhibit};
use gdk::EventType;

mod emojis;

macro_rules! die (
    ($($tt:tt),*) => ({
        eprintln!($($tt),*);
        std::process::exit(1);
    })
);

fn main() {
    let matches = {
        clap::App::new("emoji-quickpick")
        .version("1.0")
        .author("Andrew Cann")
        .arg(
            clap::Arg::with_name("input_delay")
            .short("d")
            .long("input_delay")
            .value_name("DURATION")
            .help("\
                Sets the delay in milliseconds between selecting an emoji and sending the text to \
                the X server. If this is too short then the text can get dropped. Default is \
                350ms.\
            ")
        )
        .get_matches()
    };
    let delay = match matches.value_of("input_delay") {
        Some(s) => {
            let millis: u64 = {
                FromStr::from_str(s)
                .unwrap_or_else(|e| die!("input delay must be a number: {}", e))
            };
            if millis > 2000 {
                die!("delay must be 2000ms or less");
            };
            Duration::from_millis(millis)
        },
        None => Duration::from_millis(350),
    };

    let xdo = unsafe {
        libxdo_sys::xdo_new(ptr::null())
    };
    let mut xwindow = 0;
    let err = unsafe {
        libxdo_sys::xdo_get_focused_window(xdo, &mut xwindow as *mut _)
    };
    if err != 0 {
        die!("xdo_get_focused_window failed (error {})", err);
    }


    let application = {
        Application::new("org.canndrew.emoji-quickpick", Default::default())
        .unwrap_or_else(|e| die!("Failed to create gtk application: {}", e))
    };

    let emoji = Rc::new(RefCell::new(None));
    let emoji_tx = emoji.clone();
    application.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("emoji-quickpick");
        window.set_modal(true);
        window.set_resizable(false);
        window.set_position(WindowPosition::Center);
        window.connect_event(|window, event| {
            if let Some(0xff1b) = event.get_keyval() {
                window.close()
            }
            Default::default()
        });

        let v_box = gtk::Box::new(Orientation::Vertical, 0);

        let entry = SearchEntry::new();
        v_box.pack_start(&entry, true, true, 0);

        let list_box = ListBox::new();
        let list_store = gio::ListStore::new(Label::static_type());
        let list_box_cloned = list_box.clone();
        list_box.bind_model(&list_store, move |item| {
            let item = item.downcast_ref::<Label>().unwrap();
            let list_box_row = ListBoxRow::new();
            list_box_row.add(item);

            if list_box_cloned.get_selected_row().is_none() {
                list_box_cloned.select_row(Some(&list_box_row));
            }

            list_box_row.show_all();
            list_box_row.upcast::<Widget>()
        });
        v_box.pack_start(&list_box, false, false, 0);

        window.add(&v_box);

        entry.connect_changed(move |entry| {
            let text = entry.get_text().unwrap();
            let text = text.as_str();

            let mut matches = Vec::new();
            for (emoji, name) in emojis::EMOJIS {
                if let Some(result) = sublime_fuzzy::best_match(text, name) {
                    let score = result.score();
                    match matches.iter().position(|(this_score, _, _)| {
                        *this_score < score
                    }) {
                        Some(i) => {
                            matches.insert(i, (score, emoji, name));
                        }
                        None => {
                            matches.push((score, emoji, name));
                        },
                    };
                    if matches.len() > 5 {
                        let _ = matches.pop();
                    }
                }
            }

            list_store.remove_all();
            for (_score, emoji, name) in matches {
                let text = format!("{} ({})", name, emoji);
                list_store.append(&Label::new(&*text));
            }
        });

        let list_box_cloned = list_box.clone();
        let emoji_tx = emoji_tx.clone();
        let window_cloned = window.clone();
        entry.connect_event(move |_entry, event| {
            if event.get_event_type() == EventType::KeyPress {
                let keyval = event.get_keyval().unwrap();
                if 0xff52 == keyval { // Up
                    let mut index = 0;
                    let mut previous = None;
                    while let Some(row) = list_box_cloned.get_row_at_index(index) {
                        if row.is_selected() {
                            if let Some(previous_row) = previous {
                                list_box_cloned.select_row(Some(&previous_row));
                            }
                            break;
                        }
                        previous = Some(row.clone());
                        index += 1;
                    }
                    return Inhibit(true);
                }
                if 0xff54 == keyval { // Down
                    let mut index = 0;
                    while let Some(row) = list_box_cloned.get_row_at_index(index) {
                        if row.is_selected() {
                            if let Some(next_row) = list_box_cloned.get_row_at_index(index + 1) {
                                list_box_cloned.select_row(Some(&next_row));
                            }
                            break;
                        }
                        index += 1;
                    }
                    return Inhibit(true);
                }
                if 0xff89 == keyval { // Tab
                    return Inhibit(true);
                }
                if 0xff8d == keyval || 0xff0d == keyval { // Enter and Return
                    let mut index = 0;
                    while let Some(row) = list_box_cloned.get_row_at_index(index) {
                        if row.is_selected() {
                            let mut labels = row.get_children();
                            assert_eq!(labels.len(), 1);
                            let label = labels.remove(0);
                            let label = label.downcast_ref::<Label>().unwrap();
                            let text = label.get_text().unwrap();
                            let text = text.as_str();
                            let start_pos = text.rfind('(').unwrap() + 1;
                            let end_pos = text.rfind(')').unwrap();
                            emoji_tx.replace(Some(String::from(&text[start_pos..end_pos])));
                            window_cloned.close();
                            break;
                        }
                        index += 1;
                    }
                }
            }
            Default::default()
        });
        window.show_all();
    });

    application.run(&[]);

    let emoji = emoji.borrow();
    if let Some(text) = &*emoji {
        thread::sleep(delay);
        let text = CString::new(&**text).unwrap();
        let err = unsafe {
            libxdo_sys::xdo_enter_text_window(xdo, xwindow, text.as_ptr(), delay.as_micros() as u32)
        };
        if err != 0 {
            die!("xdo_enter_text_window failed (error {})", err);
        }
        unsafe {
            libxdo_sys::xdo_free(xdo);
        }
    }
    thread::sleep(delay);
}

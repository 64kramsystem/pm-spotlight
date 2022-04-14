use fltk::image::PngImage;
use phf::phf_map;

use super::searcher::Searcher;
use crate::helpers::clipboard_management::copy_to_clipboard;

const EMOJI_PATTERNS: phf::Map<&str, &str> = phf_map! {
    "+1, thumbs up"                                           => "👍",
    "thumbs down"                                             => "👎",
    "angry"                                                   => "😠",
    "angry_cursing"                                           => "🤬",
    "anguished, anxious, scared"                              => "😧",
    "anxious_face_with_sweat, sick"                           => "😰",
    "astonished"                                              => "😲",
    "bald"                                                    => "🦲",
    "beaming_face_with_smiling_eyes, grin"                    => "😁",
    "birthday_cake"                                           => "🎂",
    "blinking"                                                => "😉",
    "blush"                                                   => "😊",
    "bouquet, flowers"                                        => "💐",
    "butt, ass"                                               => "🍑",
    "child"                                                   => "🧒",
    "boy, child"                                              => "👦",
    "girl, child"                                             => "👧",
    "clinking_glasses, party, drink"                          => "🥂",
    "cocktail_glass, drink"                                   => "🍸",
    "confetti_ball, party"                                    => "🎊",
    "confused, puzzled, baffled"                              => "😕",
    "cold_face, freezing"                                     => "🥶",
    "cool, sunglasses"                                        => "😎",
    "crossed_fingers"                                         => "🤞",
    "crying, tear"                                            => "😢",
    "disappointed, sad"                                       => "😞",
    "duck"                                                    => "🦆",
    "clown"                                                   => "🤡",
    "colbert, face with raised eyebrow"                       => "🤨",
    "dancer, woman dancing"                                   => "💃",
    "dizzy, dead"                                             => "😵",
    "drooling, salivating"                                    => "🤤",
    "exploding_head"                                          => "🤯",
    "face_with_spiral_eyes, dizzy, hypnotized"                => "😵‍💫",
    "fact_with_monocle, eyeglasses"                           => "🧐",
    "face_with_open_mouth, jaw drop"                          => "😮",
    "face_with_rolling_eyes"                                  => "🙄",
    "face_with_steam, fight"                                  => "😤",
    "fart"                                                    => "💨",
    "fearful_face, anxious"                                   => "😨",
    "fireworks"                                               => "🎆",
    "flexed_bicep, muscle"                                    => "💪",
    "flushed, embarrassed"                                    => "😳",
    "folded_hands, pray"                                      => "🙏",
    "grimacing_face"                                          => "😬",
    "grinning_face_with_big_eyes, happy"                      => "😃",
    "grinning_face_with_sweat, cold sweat"                    => "😅",
    "guitar"                                                  => "🎸",
    "hamburger"                                               => "🍔",
    "happy_devil, evil"                                       => "😈",
    "heart"                                                   => "❤️",
    "heart_eyes"                                              => "😍",
    "hot_face"                                                => "🥵",
    "hugging"                                                 => "🤗",
    "hushed, surprise, confused"                              => "😯",
    "ice_skate"                                               => "⛸️",
    "laughing"                                                => "😆",
    "laughing (joy)"                                          => "😂",
    "laughing_with_hand"                                      => "🤭",
    "lightbulb"                                               => "💡",
    "loudly_crying_face, crying river"                        => "😭",
    "man"                                                     => "👨",
    "man_dancing, man dancer"                                 => "🕺",
    "middle_finger"                                           => "🖕",
    "money_mouth_face"                                        => "🤑",
    "nerd, eyeglasses"                                        => "🤓",
    "neutral_face"                                            => "😐",
    "party_face"                                              => "🥳",
    "party_popper"                                            => "🎉",
    "pensive, sadder"                                         => "😔",
    "person_facepalming"                                      => "🤦",
    "person_raising_hand, greeting"                           => "🙋",
    "person_gesturing_no"                                     => "🙅",
    "person_gesturing_ok, yes"                                => "🙆",
    "person_bowing, pray, sorry"                              => "🙇",
    "pistol, gun"                                             => "🔫",
    "pizza"                                                   => "🍕",
    "pleading, big eyes, cute"                                => "🥺",
    "princess"                                                => "👸",
    "relaxed"                                                 => "☺",
    "relieved"                                                => "😌",
    "rolling_on_the_floor, rotfl"                             => "🤣",
    "slightly_frowning, sad"                                  => "🙁",
    "screaming"                                               => "😱",
    "see_no_evil, monkey, facepalm"                           => "🙈",
    "shit"                                                    => "💩",
    "shrug"                                                   => "🤷",
    "face_with_bandage, sickness, operation, surgery, injury" => "🤕",
    "sign_of_the_horns, metal"                                => "🤘",
    "skier"                                                   => "⛷️",
    "skull, dead"                                             => "💀",
    "sleepy"                                                  => "😴",
    "slightly_smiling_face"                                   => "🙂",
    "smiling"                                                 => "😄",
    "smiling_face_with_halo"                                  => "😇",
    "smiling_face_with_tear"                                  => "🥲",
    "smirk"                                                   => "😏",
    "sneezing_face"                                           => "🤧",
    "superhero"                                               => "🦸‍♂️",
    "star_struck, starry eyes"                                => "🤩",
    "thermometer"                                             => "🌡️",
    "thinking"                                                => "🤔",
    "throwing_up"                                             => "🤮",
    "tropical_drink"                                          => "🍹",
    "unamused"                                                => "😒",
    "upside-down"                                             => "🙃",
    "waving_hand"                                             => "👋",
    "without mouth, speechless, mute"                         => "😶",
    "wink"                                                    => "😉",
    "winking_face_with_tongue"                                => "😜",
    "woman"                                                   => "👩",
    "woozy, zany"                                             => "🥴",
    "yum, food, hungry, slurp, full belly"                    => "😋",
    "zany_face, o_O"                                          => "🤪",
    "zipper_mouth"                                            => "🤐",
    "zombie"                                                  => "🧟",
};

pub struct EmojiSearcher {}

impl EmojiSearcher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Searcher for EmojiSearcher {
    fn handles(&self, pattern: &str) -> bool {
        pattern.starts_with(":")
    }

    fn search(&mut self, pattern: &str) -> Vec<(Option<PngImage>, String, Option<String>)> {
        let pattern = pattern.chars().skip(1).collect::<String>();

        if pattern.len() > 0 {
            EMOJI_PATTERNS
                .into_iter()
                .filter_map(|(patterns, emoji)| {
                    if patterns.contains(&pattern) {
                        Some((None, patterns.to_string(), Some(emoji.to_string())))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn execute(&self, emoji: String) {
        copy_to_clipboard(emoji);
    }
}

use fltk::image::PngImage;
use phf::phf_map;

use super::searcher::Searcher;
use crate::helpers::clipboard_management::copy_to_clipboard;

const EMOJI_ICON_PATTERNS: phf::Map<&str, (&str, &[u8])> = phf_map! {
    "👍" => ("+1, thumbs up",                                           include_bytes!("../../resources/emoji_icons/thumbs_up.png")),
    "👎" => ("-1, thumbs down",                                         include_bytes!("../../resources/emoji_icons/thumbs_down.png")),
    "😠" => ("angry",                                                   include_bytes!("../../resources/emoji_icons/angry.png")),
    "🤬" => ("angry_cursing",                                           include_bytes!("../../resources/emoji_icons/angry_cursing.png")),
    "😧" => ("anguished, anxious, scared",                              include_bytes!("../../resources/emoji_icons/anguished.png")),
    "😰" => ("anxious_face_with_sweat, sick",                           include_bytes!("../../resources/emoji_icons/anxious_with_sweat.png")),
    "😲" => ("astonished",                                              include_bytes!("../../resources/emoji_icons/astonished.png")),
    "🦲" => ("bald",                                                    include_bytes!("../../resources/emoji_icons/bald.png")),
    "😁" => ("beaming_face_with_smiling_eyes, grin",                    include_bytes!("../../resources/emoji_icons/beaming.png")),
    "🎂" => ("birthday_cake",                                           include_bytes!("../../resources/emoji_icons/birthday_cake.png")),
    "😊" => ("blush",                                                   include_bytes!("../../resources/emoji_icons/blush.png")),
    "💐" => ("bouquet, flowers",                                        include_bytes!("../../resources/emoji_icons/bouquet.png")),
    "🍑" => ("butt, ass",                                               include_bytes!("../../resources/emoji_icons/butt.png")),
    "🧒" => ("child",                                                   include_bytes!("../../resources/emoji_icons/child.png")),
    "👦" => ("boy, child",                                              include_bytes!("../../resources/emoji_icons/boy.png")),
    "👧" => ("girl, child",                                             include_bytes!("../../resources/emoji_icons/girl.png")),
    "🥂" => ("clinking_glasses, party, drink",                          include_bytes!("../../resources/emoji_icons/glasses.png")),
    "🍸" => ("cocktail_glass, drink",                                   include_bytes!("../../resources/emoji_icons/cocktail_glass.png")),
    "🎊" => ("confetti_ball, party",                                    include_bytes!("../../resources/emoji_icons/confetti_ball.png")),
    "😕" => ("confused, puzzled, baffled",                              include_bytes!("../../resources/emoji_icons/confused.png")),
    "🥶" => ("cold_face, freezing",                                     include_bytes!("../../resources/emoji_icons/cold.png")),
    "😎" => ("cool, sunglasses",                                        include_bytes!("../../resources/emoji_icons/cool.png")),
    "🤞" => ("crossed_fingers",                                         include_bytes!("../../resources/emoji_icons/crossed_fingers.png")),
    "😢" => ("crying, tear",                                            include_bytes!("../../resources/emoji_icons/crying.png")),
    "😞" => ("disappointed, sad",                                       include_bytes!("../../resources/emoji_icons/disappointed.png")),
    "🦆" => ("duck",                                                    include_bytes!("../../resources/emoji_icons/duck.png")),
    "🤡" => ("clown",                                                   include_bytes!("../../resources/emoji_icons/clown.png")),
    "🤨" => ("colbert, face with raised eyebrow",                       include_bytes!("../../resources/emoji_icons/colbert.png")),
    "💃" => ("dancer, woman dancing",                                   include_bytes!("../../resources/emoji_icons/dancer_woman.png")),
    "😵" => ("dizzy, dead",                                             include_bytes!("../../resources/emoji_icons/dizzy.png")),
    "🤤" => ("drooling, salivating",                                    include_bytes!("../../resources/emoji_icons/drooling.png")),
    "🤯" => ("exploding_head",                                          include_bytes!("../../resources/emoji_icons/exploding_head.png")),
    "😵‍💫" => ("face_with_spiral_eyes, dizzy, hypnotized",                include_bytes!("../../resources/emoji_icons/spiral_eyes.png")),
    "🧐" => ("face_with_monocle, eyeglasses",                           include_bytes!("../../resources/emoji_icons/monocle.png")),
    "😮" => ("face_with_open_mouth, jaw drop",                          include_bytes!("../../resources/emoji_icons/open_mouth.png")),
    "🙄" => ("face_with_rolling_eyes",                                  include_bytes!("../../resources/emoji_icons/princess.png")),
    "😤" => ("face_with_steam, fight",                                  include_bytes!("../../resources/emoji_icons/princess.png")),
    "💨" => ("fart",                                                    include_bytes!("../../resources/emoji_icons/princess.png")),
    "😨" => ("fearful_face, anxious",                                   include_bytes!("../../resources/emoji_icons/princess.png")),
    "🎆" => ("fireworks",                                               include_bytes!("../../resources/emoji_icons/princess.png")),
    "💪" => ("flexed_bicep, muscle",                                    include_bytes!("../../resources/emoji_icons/princess.png")),
    "😳" => ("flushed, embarrassed",                                    include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙏" => ("folded_hands, pray",                                      include_bytes!("../../resources/emoji_icons/princess.png")),
    "😬" => ("grimacing_face",                                          include_bytes!("../../resources/emoji_icons/princess.png")),
    "😃" => ("grinning_face_with_big_eyes, happy",                      include_bytes!("../../resources/emoji_icons/princess.png")),
    "😅" => ("grinning_face_with_sweat, cold sweat",                    include_bytes!("../../resources/emoji_icons/princess.png")),
    "🎸" => ("guitar",                                                  include_bytes!("../../resources/emoji_icons/princess.png")),
    "🍔" => ("hamburger",                                               include_bytes!("../../resources/emoji_icons/princess.png")),
    "😈" => ("happy_devil, evil",                                       include_bytes!("../../resources/emoji_icons/princess.png")),
    "❤️" => ("heart",                                                   include_bytes!("../../resources/emoji_icons/princess.png")),
    "😍" => ("heart_eyes",                                              include_bytes!("../../resources/emoji_icons/princess.png")),
    "🥵" => ("hot_face",                                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤗" => ("hugging",                                                 include_bytes!("../../resources/emoji_icons/princess.png")),
    "😯" => ("hushed, surprise, confused",                              include_bytes!("../../resources/emoji_icons/princess.png")),
    "⛸️" => ("ice_skate",                                               include_bytes!("../../resources/emoji_icons/princess.png")),
    "😆" => ("laughing",                                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "😂" => ("laughing (joy)",                                          include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤭" => ("laughing_with_hand",                                      include_bytes!("../../resources/emoji_icons/princess.png")),
    "💡" => ("lightbulb",                                               include_bytes!("../../resources/emoji_icons/princess.png")),
    "😭" => ("loudly_crying_face, crying river",                        include_bytes!("../../resources/emoji_icons/princess.png")),
    "👨" => ("man",                                                     include_bytes!("../../resources/emoji_icons/princess.png")),
    "🕺" => ("man_dancing, man dancer",                                 include_bytes!("../../resources/emoji_icons/princess.png")),
    "🖕" => ("middle_finger",                                           include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤑" => ("money_mouth_face",                                        include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤓" => ("nerd, eyeglasses",                                        include_bytes!("../../resources/emoji_icons/princess.png")),
    "😐" => ("neutral_face",                                            include_bytes!("../../resources/emoji_icons/princess.png")),
    "🥳" => ("party_face",                                              include_bytes!("../../resources/emoji_icons/princess.png")),
    "🎉" => ("party_popper",                                            include_bytes!("../../resources/emoji_icons/princess.png")),
    "😔" => ("pensive, sadder",                                         include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤦" => ("person_facepalming",                                      include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙋" => ("person_raising_hand, greeting",                           include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙅" => ("person_gesturing_no",                                     include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙆" => ("person_gesturing_ok, yes",                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙇" => ("person_bowing, pray, sorry",                              include_bytes!("../../resources/emoji_icons/princess.png")),
    "🔫" => ("pistol, gun",                                             include_bytes!("../../resources/emoji_icons/princess.png")),
    "🍕" => ("pizza",                                                   include_bytes!("../../resources/emoji_icons/princess.png")),
    "🥺" => ("pleading, big eyes, cute",                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "👸" => ("princess",                                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "☺" => ("relaxed",                                                 include_bytes!("../../resources/emoji_icons/princess.png")),
    "😌" => ("relieved",                                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤣" => ("rolling_on_the_floor, rotfl",                             include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙁" => ("slightly_frowning, sad",                                  include_bytes!("../../resources/emoji_icons/princess.png")),
    "😱" => ("screaming",                                               include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙈" => ("see_no_evil, monkey",                                     include_bytes!("../../resources/emoji_icons/princess.png")),
    "💩" => ("shit",                                                    include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤷" => ("shrug",                                                   include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤕" => ("face_with_bandage, sickness, operation, surgery, injury", include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤘" => ("sign_of_the_horns, metal",                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "⛷️" => ("skier",                                                   include_bytes!("../../resources/emoji_icons/princess.png")),
    "💀" => ("skull, dead",                                             include_bytes!("../../resources/emoji_icons/princess.png")),
    "😴" => ("sleepy",                                                  include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙂" => ("slightly_smiling_face",                                   include_bytes!("../../resources/emoji_icons/princess.png")),
    "😄" => ("smiling",                                                 include_bytes!("../../resources/emoji_icons/princess.png")),
    "😇" => ("smiling_face_with_halo",                                  include_bytes!("../../resources/emoji_icons/princess.png")),
    "🥲" => ("smiling_face_with_tear",                                  include_bytes!("../../resources/emoji_icons/princess.png")),
    "😏" => ("smirk",                                                   include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤧" => ("sneezing_face",                                           include_bytes!("../../resources/emoji_icons/princess.png")),
    "🦸‍♂️" => ("superhero",                                               include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤩" => ("star_struck, starry eyes",                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "🌡️" => ("thermometer",                                             include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤔" => ("thinking",                                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤮" => ("throwing_up",                                             include_bytes!("../../resources/emoji_icons/princess.png")),
    "🍹" => ("tropical_drink",                                          include_bytes!("../../resources/emoji_icons/princess.png")),
    "😒" => ("unamused",                                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "🙃" => ("upside-down",                                             include_bytes!("../../resources/emoji_icons/princess.png")),
    "👋" => ("waving_hand",                                             include_bytes!("../../resources/emoji_icons/princess.png")),
    "😶" => ("without mouth, speechless, mute",                         include_bytes!("../../resources/emoji_icons/princess.png")),
    "😉" => ("wink",                                                    include_bytes!("../../resources/emoji_icons/princess.png")),
    "😜" => ("winking_face_with_tongue",                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "👩" => ("woman",                                                   include_bytes!("../../resources/emoji_icons/princess.png")),
    "🥴" => ("woozy, zany",                                             include_bytes!("../../resources/emoji_icons/princess.png")),
    "😋" => ("yum, food, hungry, slurp, full belly",                    include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤪" => ("zany_face, o_O",                                          include_bytes!("../../resources/emoji_icons/princess.png")),
    "🤐" => ("zipper_mouth",                                            include_bytes!("../../resources/emoji_icons/princess.png")),
    "🧟" => ("zombie",                                                  include_bytes!("../../resources/emoji_icons/princess.png")),
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
            EMOJI_ICON_PATTERNS
                .into_iter()
                .filter_map(|(emoji, (patterns, image_bytes))| {
                    if patterns.contains(&pattern) {
                        Some((
                            PngImage::from_data(image_bytes).ok(),
                            patterns.to_string(),
                            Some(emoji.to_string()),
                        ))
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

use fltk::{app::Sender, image::PngImage};
use phf::phf_map;
use std::process;

use super::{search_result_entry::SearchResultEntry, searcher::Searcher};
use crate::{
    gui::message_event::MessageEvent::{self, UpdateList},
    helpers::clipboard_management::copy_to_clipboard,
};

// The reference for the Emoji is Emojipedia.
//
// The emoji characters (hash keys) are Emojipedia reference image; the images seem to have been extracted
// from different sets.
//
// Some emojis have been unnecessarily generated from a character, but there's no value in regenerating them.
//
// Conversion commands:
//
//     ICON_NAME=enter
//     # This is only for generating images from characters
//     echo -e "↵" | pango-view --dpi=300 --no-display --font='Droid Sans Mono' --output=emoji_icons.source/$ICON_NAME.png /dev/stdin
//     convert emoji_icons.source/$ICON_NAME.png -resize 30x30 emoji_icons/$ICON_NAME.png
//
const EMOJI_ICON_PATTERNS: phf::Map<&str, (&str, &[u8])> = phf_map! {
    // Emojis (symbols are at the bottom)
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
    "😘" => ("blowing_kiss",                                            include_bytes!("../../resources/emoji_icons/blowing_kiss.png")),
    "😊" => ("blush",                                                   include_bytes!("../../resources/emoji_icons/blush.png")),
    "💐" => ("bouquet, flowers",                                        include_bytes!("../../resources/emoji_icons/bouquet.png")),
    "👦" => ("boy, child",                                              include_bytes!("../../resources/emoji_icons/boy.png")),
    "🍑" => ("butt, ass",                                               include_bytes!("../../resources/emoji_icons/butt.png")),
    "🤙" => ("call_me",                                                 include_bytes!("../../resources/emoji_icons/call_me.png")), // from char
    "🤌" => ("cazzuvo",                                                 include_bytes!("../../resources/emoji_icons/cazzuvo.png")), // from char
    "🧒" => ("child",                                                   include_bytes!("../../resources/emoji_icons/child.png")),
    "🥂" => ("clinking_glasses, party, drink",                          include_bytes!("../../resources/emoji_icons/glasses.png")),
    "🤡" => ("clown",                                                   include_bytes!("../../resources/emoji_icons/clown.png")),
    "🍸" => ("cocktail_glass, drink",                                   include_bytes!("../../resources/emoji_icons/cocktail_glass.png")),
    "🤨" => ("colbert, face with raised eyebrow",                       include_bytes!("../../resources/emoji_icons/colbert.png")),
    "🥶" => ("cold_face, freezing",                                     include_bytes!("../../resources/emoji_icons/cold.png")),
    "🎊" => ("confetti_ball, party",                                    include_bytes!("../../resources/emoji_icons/confetti_ball.png")),
    "😕" => ("confused, puzzled, baffled",                              include_bytes!("../../resources/emoji_icons/confused.png")),
    "😎" => ("cool, sunglasses",                                        include_bytes!("../../resources/emoji_icons/cool.png")),
    "🤞" => ("crossed_fingers",                                         include_bytes!("../../resources/emoji_icons/crossed_fingers.png")),
    "😢" => ("crying, tear",                                            include_bytes!("../../resources/emoji_icons/crying.png")),
    "💃" => ("dancer, woman dancing",                                   include_bytes!("../../resources/emoji_icons/dancer_woman.png")),
    "😞" => ("disappointed, sad",                                       include_bytes!("../../resources/emoji_icons/disappointed.png")),
    "😵" => ("dizzy, dead",                                             include_bytes!("../../resources/emoji_icons/dizzy.png")),
    "🤤" => ("drooling, salivating",                                    include_bytes!("../../resources/emoji_icons/drooling.png")),
    "🦆" => ("duck",                                                    include_bytes!("../../resources/emoji_icons/duck.png")),
    "🤯" => ("exploding_head",                                          include_bytes!("../../resources/emoji_icons/exploding_head.png")),
    "🤕" => ("face_with_bandage, sickness, operation, surgery, injury", include_bytes!("../../resources/emoji_icons/bandaged.png")),
    "🧐" => ("face_with_monocle, eyeglasses",                           include_bytes!("../../resources/emoji_icons/monocle.png")),
    "😮" => ("face_with_open_mouth, jaw drop",                          include_bytes!("../../resources/emoji_icons/open_mouth.png")),
    "🙄" => ("face_with_rolling_eyes",                                  include_bytes!("../../resources/emoji_icons/rolling_eyes.png")),
    "😵‍💫" => ("face_with_spiral_eyes, dizzy, hypnotized",                include_bytes!("../../resources/emoji_icons/spiral_eyes.png")),
    "😤" => ("face_with_steam, fight",                                  include_bytes!("../../resources/emoji_icons/steam_nose.png")),
    "🤒" => ("face_with_thermometer, sick",                             include_bytes!("../../resources/emoji_icons/face_with_thermometer.png")),
    "💨" => ("fart",                                                    include_bytes!("../../resources/emoji_icons/fart.png")),
    "😨" => ("fearful_face, anxious",                                   include_bytes!("../../resources/emoji_icons/fearful.png")),
    "🎆" => ("fireworks",                                               include_bytes!("../../resources/emoji_icons/fireworks.png")),
    "💪" => ("flexed_bicep, muscle",                                    include_bytes!("../../resources/emoji_icons/muscle.png")),
    "😳" => ("flushed, embarrassed",                                    include_bytes!("../../resources/emoji_icons/flushed.png")),
    "🙏" => ("folded_hands, pray",                                      include_bytes!("../../resources/emoji_icons/pray.png")),
    "👧" => ("girl, child",                                             include_bytes!("../../resources/emoji_icons/girl.png")),
    "😬" => ("grimacing_face",                                          include_bytes!("../../resources/emoji_icons/grimacing.png")),
    "😃" => ("grinning_face_with_big_eyes, happy",                      include_bytes!("../../resources/emoji_icons/grinning.png")),
    "😅" => ("grinning_face_with_sweat, cold sweat",                    include_bytes!("../../resources/emoji_icons/smiling_sweat.png")),
    "🎸" => ("guitar",                                                  include_bytes!("../../resources/emoji_icons/guitar.png")),
    "🍔" => ("hamburger",                                               include_bytes!("../../resources/emoji_icons/hamburger.png")),
    "😈" => ("happy_devil, evil",                                       include_bytes!("../../resources/emoji_icons/happy_devil.png")),
    "❤️" => ("heart",                                                   include_bytes!("../../resources/emoji_icons/heart.png")),
    "😍" => ("heart_eyes",                                              include_bytes!("../../resources/emoji_icons/heart_eyes.png")),
    "👩‍❤️‍👨" => ("heart_woman_man",                                         include_bytes!("../../resources/emoji_icons/heart_woman_man.png")), // from char
    "🥵" => ("hot_face, heat",                                          include_bytes!("../../resources/emoji_icons/hot_face.png")),
    "😯" => ("hushed, surprised, confused",                             include_bytes!("../../resources/emoji_icons/hushed.png")),
    "⛸️" => ("ice_skate",                                               include_bytes!("../../resources/emoji_icons/ice_skate.png")),
    "👩‍❤️‍💋‍👨" => ("kiss_woman_man",                                          include_bytes!("../../resources/emoji_icons/kiss_woman_man.png")), // from char
    "😆" => ("laughing",                                                include_bytes!("../../resources/emoji_icons/laughing.png")),
    "😂" => ("laughing (joy)",                                          include_bytes!("../../resources/emoji_icons/laughing_joy.png")),
    "🤭" => ("laughing_with_hand",                                      include_bytes!("../../resources/emoji_icons/laughing_hand.png")),
    "( ͡° ͜ʖ ͡°)" => ("lenny",                                               include_bytes!("../../resources/emoji_icons/smirking.png")),
    "💡" => ("lightbulb",                                               include_bytes!("../../resources/emoji_icons/lightbulb.png")),
    "😭" => ("loudly_crying_face, crying river",                        include_bytes!("../../resources/emoji_icons/crying_loudly.png")),
    "👨" => ("man",                                                     include_bytes!("../../resources/emoji_icons/man.png")),
    "🕺" => ("man_dancing, man dancer",                                 include_bytes!("../../resources/emoji_icons/dancing_man.png")),
    "🫠" => ("melting_face",                                            include_bytes!("../../resources/emoji_icons/melting_face.png")),
    "🖕" => ("middle_finger",                                           include_bytes!("../../resources/emoji_icons/middle_finger.png")),
    "🤑" => ("money_mouth_face",                                        include_bytes!("../../resources/emoji_icons/money_face.png")),
    "🥹" => ("moved_eyes, holding_tears",                               include_bytes!("../../resources/emoji_icons/holding_tears.png")), // from char
    "🥸" => ("mustache_nerd, disguised_face",                           include_bytes!("../../resources/emoji_icons/disguised_face.png")),
    "🤢" => ("nauseated_face, throwing_up",                             include_bytes!("../../resources/emoji_icons/nauseated_face.png")),
    "🤓" => ("nerd, eyeglasses",                                        include_bytes!("../../resources/emoji_icons/nerd.png")),
    "😐" => ("neutral_face",                                            include_bytes!("../../resources/emoji_icons/neutral.png")),
    "🥳" => ("party_face",                                              include_bytes!("../../resources/emoji_icons/party_face.png")),
    "🎉" => ("party_popper",                                            include_bytes!("../../resources/emoji_icons/party_popper.png")),
    "🫣" => ("peeking eye, hiding_face",                                include_bytes!("../../resources/emoji_icons/hiding_face.png")),
    "😔" => ("pensive, sadder",                                         include_bytes!("../../resources/emoji_icons/pensive.png")),
    "🙇" => ("person_bowing, pray, sorry",                              include_bytes!("../../resources/emoji_icons/bowing.png")),
    "🤦" => ("person_facepalming",                                      include_bytes!("../../resources/emoji_icons/facepalming.png")),
    "🙅" => ("person_gesturing_no",                                     include_bytes!("../../resources/emoji_icons/person_no.png")),
    "🙆" => ("person_gesturing_ok, yes",                                include_bytes!("../../resources/emoji_icons/person_yes.png")),
    "🙋" => ("person_raising_hand, greeting",                           include_bytes!("../../resources/emoji_icons/raising_hand.png")),
    "🔫" => ("pistol, gun",                                             include_bytes!("../../resources/emoji_icons/pistol.png")),
    "🍕" => ("pizza",                                                   include_bytes!("../../resources/emoji_icons/pizza.png")),
    "🥺" => ("pleading, big eyes, cute",                                include_bytes!("../../resources/emoji_icons/pleading.png")),
    "👸" => ("princess",                                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "☺" => ("relaxed",                                                 include_bytes!("../../resources/emoji_icons/relaxed.png")),
    "😌" => ("relieved",                                                include_bytes!("../../resources/emoji_icons/relieved.png")),
    "🤖" => ("robot",                                                   include_bytes!("../../resources/emoji_icons/robot.png")),
    "🤣" => ("rolling_on_the_floor, rotfl",                             include_bytes!("../../resources/emoji_icons/rotfl.png")),
    "😱" => ("screaming",                                               include_bytes!("../../resources/emoji_icons/screaming.png")),
    "🙈" => ("see_no_evil, monkey",                                     include_bytes!("../../resources/emoji_icons/see_no_evil.png")),
    "💩" => ("shit",                                                    include_bytes!("../../resources/emoji_icons/shit.png")),
    "🤷" => ("shrug",                                                   include_bytes!("../../resources/emoji_icons/shrugging.png")),
    "🤘" => ("sign_of_the_horns, metal",                                include_bytes!("../../resources/emoji_icons/horns.png")),
    "⛷️" => ("skier",                                                   include_bytes!("../../resources/emoji_icons/skier.png")),
    "💀" => ("skull, dead",                                             include_bytes!("../../resources/emoji_icons/skull.png")),
    "😴" => ("sleepy",                                                  include_bytes!("../../resources/emoji_icons/sleepy.png")),
    "🙁" => ("slightly_frowning, sad",                                  include_bytes!("../../resources/emoji_icons/frowning.png")),
    "🙂" => ("slightly_smiling_face",                                   include_bytes!("../../resources/emoji_icons/smiling_slightly.png")),
    "😄" => ("smiling",                                                 include_bytes!("../../resources/emoji_icons/smiling.png")),
    "😇" => ("smiling_face_with_halo",                                  include_bytes!("../../resources/emoji_icons/halo.png")),
    "🤗" => ("smiling_face_with_open_hands,hugging",                    include_bytes!("../../resources/emoji_icons/hugging.png")),
    "🥲" => ("smiling_face_with_tear",                                  include_bytes!("../../resources/emoji_icons/smiling_tear.png")),
    "😏" => ("smirk",                                                   include_bytes!("../../resources/emoji_icons/smirking.png")),
    "🤧" => ("sneezing_face",                                           include_bytes!("../../resources/emoji_icons/sneezing.png")),
    "🤩" => ("star_struck, starry eyes",                                include_bytes!("../../resources/emoji_icons/starstruck.png")),
    "🦸‍♂️" => ("superhero",                                               include_bytes!("../../resources/emoji_icons/superhero.png")),
    "🌡️" => ("thermometer",                                             include_bytes!("../../resources/emoji_icons/thermometer.png")),
    "🤔" => ("thinking",                                                include_bytes!("../../resources/emoji_icons/thinking.png")),
    "🤮" => ("throwing_up",                                             include_bytes!("../../resources/emoji_icons/vomiting.png")),
    "🍹" => ("tropical_drink",                                          include_bytes!("../../resources/emoji_icons/tropical_drink.png")),
    "😒" => ("unamused",                                                include_bytes!("../../resources/emoji_icons/unamused.png")),
    "🙃" => ("upside-down",                                             include_bytes!("../../resources/emoji_icons/upside_down.png")),
    "👋" => ("waving_hand",                                             include_bytes!("../../resources/emoji_icons/waving_hand.png")),
    "😉" => ("wink",                                                    include_bytes!("../../resources/emoji_icons/wink.png")),
    "😜" => ("winking_face_with_tongue",                                include_bytes!("../../resources/emoji_icons/wink_tongue.png")),
    "😶" => ("without mouth, speechless, mute",                         include_bytes!("../../resources/emoji_icons/mouthless.png")),
    "👩" => ("woman",                                                   include_bytes!("../../resources/emoji_icons/woman.png")),
    "🥴" => ("woozy, zany",                                             include_bytes!("../../resources/emoji_icons/woozy.png")),
    "🫡" => ("yessir, saluting_face",                                   include_bytes!("../../resources/emoji_icons/saluting_face.png")),
    "🥱" => ("yawning_face",                                            include_bytes!("../../resources/emoji_icons/yawning_face.png")),
    "😋" => ("yum, food, hungry, slurp, full belly",                    include_bytes!("../../resources/emoji_icons/yummy.png")),
    "🤪" => ("zany_face, wacky, o_O",                                   include_bytes!("../../resources/emoji_icons/zany.png")),
    "🤐" => ("zipper_mouth",                                            include_bytes!("../../resources/emoji_icons/zipper_mouth.png")),
    "🧟" => ("zombie",                                                  include_bytes!("../../resources/emoji_icons/zombie.png")),
    // Symbols
    "↵" => ("enter",                                                   include_bytes!("../../resources/emoji_icons/enter.png")),
};

pub struct EmojiSearcher {}

impl EmojiSearcher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Searcher for EmojiSearcher {
    fn handles(&self, pattern: &str) -> bool {
        pattern.starts_with(':')
    }

    fn search(&mut self, pattern: String, sender: Sender<MessageEvent>, search_id: u32) {
        let pattern = pattern.chars().skip(1).collect::<String>();

        if !pattern.is_empty() {
            let search_result = EMOJI_ICON_PATTERNS
                .into_iter()
                .filter_map(|(emoji, (patterns, image_bytes))| {
                    if patterns.contains(&pattern) {
                        Some(SearchResultEntry::new(
                            PngImage::from_data(image_bytes).ok(),
                            patterns.to_string(),
                            Some(emoji.to_string()),
                            search_id,
                            true,
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            sender.send(UpdateList(search_result));
        }
    }

    fn execute(&self, emoji: String) -> Result<(), String> {
        copy_to_clipboard(emoji).map_err(|error| format!("Could not copy emoji: {error}"))?;
        process::exit(0);
    }
}

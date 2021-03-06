use fltk::{
    app::Sender,
    image::{PngImage, SharedImage},
};
use phf::phf_map;
use std::process;

use super::{search_result_entry::SearchResultEntry, searcher::Searcher};
use crate::{
    gui::message_event::MessageEvent::{self, UpdateList},
    helpers::clipboard_management::copy_to_clipboard,
};

// This is the Google set, from Emojipedia.
//
// Conversion command:
//
//     convert emoji_icons.source/robot.png -resize 30x30 emoji_icons/robot.png
//
const EMOJI_ICON_PATTERNS: phf::Map<&str, (&str, &[u8])> = phf_map! {
    "ð" => ("+1, thumbs up",                                           include_bytes!("../../resources/emoji_icons/thumbs_up.png")),
    "ð" => ("-1, thumbs down",                                         include_bytes!("../../resources/emoji_icons/thumbs_down.png")),
    "ð " => ("angry",                                                   include_bytes!("../../resources/emoji_icons/angry.png")),
    "ðĪŽ" => ("angry_cursing",                                           include_bytes!("../../resources/emoji_icons/angry_cursing.png")),
    "ð§" => ("anguished, anxious, scared",                              include_bytes!("../../resources/emoji_icons/anguished.png")),
    "ð°" => ("anxious_face_with_sweat, sick",                           include_bytes!("../../resources/emoji_icons/anxious_with_sweat.png")),
    "ðē" => ("astonished",                                              include_bytes!("../../resources/emoji_icons/astonished.png")),
    "ðĶē" => ("bald",                                                    include_bytes!("../../resources/emoji_icons/bald.png")),
    "ð" => ("beaming_face_with_smiling_eyes, grin",                    include_bytes!("../../resources/emoji_icons/beaming.png")),
    "ð" => ("birthday_cake",                                           include_bytes!("../../resources/emoji_icons/birthday_cake.png")),
    "ð" => ("blush",                                                   include_bytes!("../../resources/emoji_icons/blush.png")),
    "ð" => ("bouquet, flowers",                                        include_bytes!("../../resources/emoji_icons/bouquet.png")),
    "ð" => ("butt, ass",                                               include_bytes!("../../resources/emoji_icons/butt.png")),
    "ð§" => ("child",                                                   include_bytes!("../../resources/emoji_icons/child.png")),
    "ðĶ" => ("boy, child",                                              include_bytes!("../../resources/emoji_icons/boy.png")),
    "ð§" => ("girl, child",                                             include_bytes!("../../resources/emoji_icons/girl.png")),
    "ðĨ" => ("clinking_glasses, party, drink",                          include_bytes!("../../resources/emoji_icons/glasses.png")),
    "ðļ" => ("cocktail_glass, drink",                                   include_bytes!("../../resources/emoji_icons/cocktail_glass.png")),
    "ð" => ("confetti_ball, party",                                    include_bytes!("../../resources/emoji_icons/confetti_ball.png")),
    "ð" => ("confused, puzzled, baffled",                              include_bytes!("../../resources/emoji_icons/confused.png")),
    "ðĨķ" => ("cold_face, freezing",                                     include_bytes!("../../resources/emoji_icons/cold.png")),
    "ð" => ("cool, sunglasses",                                        include_bytes!("../../resources/emoji_icons/cool.png")),
    "ðĪ" => ("crossed_fingers",                                         include_bytes!("../../resources/emoji_icons/crossed_fingers.png")),
    "ðĒ" => ("crying, tear",                                            include_bytes!("../../resources/emoji_icons/crying.png")),
    "ð" => ("disappointed, sad",                                       include_bytes!("../../resources/emoji_icons/disappointed.png")),
    "ðĶ" => ("duck",                                                    include_bytes!("../../resources/emoji_icons/duck.png")),
    "ðĪĄ" => ("clown",                                                   include_bytes!("../../resources/emoji_icons/clown.png")),
    "ðĪĻ" => ("colbert, face with raised eyebrow",                       include_bytes!("../../resources/emoji_icons/colbert.png")),
    "ð" => ("dancer, woman dancing",                                   include_bytes!("../../resources/emoji_icons/dancer_woman.png")),
    "ðĩ" => ("dizzy, dead",                                             include_bytes!("../../resources/emoji_icons/dizzy.png")),
    "ðĪĪ" => ("drooling, salivating",                                    include_bytes!("../../resources/emoji_icons/drooling.png")),
    "ðĪŊ" => ("exploding_head",                                          include_bytes!("../../resources/emoji_icons/exploding_head.png")),
    "ðĩâðŦ" => ("face_with_spiral_eyes, dizzy, hypnotized",                include_bytes!("../../resources/emoji_icons/spiral_eyes.png")),
    "ð§" => ("face_with_monocle, eyeglasses",                           include_bytes!("../../resources/emoji_icons/monocle.png")),
    "ðŪ" => ("face_with_open_mouth, jaw drop",                          include_bytes!("../../resources/emoji_icons/open_mouth.png")),
    "ð" => ("face_with_rolling_eyes",                                  include_bytes!("../../resources/emoji_icons/rolling_eyes.png")),
    "ðĪ" => ("face_with_steam, fight",                                  include_bytes!("../../resources/emoji_icons/steam_nose.png")),
    "ðĻ" => ("fart",                                                    include_bytes!("../../resources/emoji_icons/fart.png")),
    "ðĻ" => ("fearful_face, anxious",                                   include_bytes!("../../resources/emoji_icons/fearful.png")),
    "ð" => ("fireworks",                                               include_bytes!("../../resources/emoji_icons/fireworks.png")),
    "ðŠ" => ("flexed_bicep, muscle",                                    include_bytes!("../../resources/emoji_icons/muscle.png")),
    "ðģ" => ("flushed, embarrassed",                                    include_bytes!("../../resources/emoji_icons/flushed.png")),
    "ð" => ("folded_hands, pray",                                      include_bytes!("../../resources/emoji_icons/pray.png")),
    "ðŽ" => ("grimacing_face",                                          include_bytes!("../../resources/emoji_icons/grimacing.png")),
    "ð" => ("grinning_face_with_big_eyes, happy",                      include_bytes!("../../resources/emoji_icons/grinning.png")),
    "ð" => ("grinning_face_with_sweat, cold sweat",                    include_bytes!("../../resources/emoji_icons/smiling_sweat.png")),
    "ðļ" => ("guitar",                                                  include_bytes!("../../resources/emoji_icons/guitar.png")),
    "ð" => ("hamburger",                                               include_bytes!("../../resources/emoji_icons/hamburger.png")),
    "ð" => ("happy_devil, evil",                                       include_bytes!("../../resources/emoji_icons/happy_devil.png")),
    "âĪïļ" => ("heart",                                                   include_bytes!("../../resources/emoji_icons/heart.png")),
    "ð" => ("heart_eyes",                                              include_bytes!("../../resources/emoji_icons/heart_eyes.png")),
    "ðĨĩ" => ("hot_face, heat",                                          include_bytes!("../../resources/emoji_icons/hot_face.png")),
    "ðĪ" => ("greeting_hands",                                          include_bytes!("../../resources/emoji_icons/greeting_hands.png")),
    "ðŊ" => ("hushed, surprised, confused",                             include_bytes!("../../resources/emoji_icons/hushed.png")),
    "âļïļ" => ("ice_skate",                                               include_bytes!("../../resources/emoji_icons/ice_skate.png")),
    "ð" => ("laughing",                                                include_bytes!("../../resources/emoji_icons/laughing.png")),
    "ð" => ("laughing (joy)",                                          include_bytes!("../../resources/emoji_icons/laughing_joy.png")),
    "ðĪ­" => ("laughing_with_hand",                                      include_bytes!("../../resources/emoji_icons/laughing_hand.png")),
    "ðĄ" => ("lightbulb",                                               include_bytes!("../../resources/emoji_icons/lightbulb.png")),
    "ð­" => ("loudly_crying_face, crying river",                        include_bytes!("../../resources/emoji_icons/crying_loudly.png")),
    "ðĻ" => ("man",                                                     include_bytes!("../../resources/emoji_icons/man.png")),
    "ðš" => ("man_dancing, man dancer",                                 include_bytes!("../../resources/emoji_icons/dancing_man.png")),
    "ð" => ("middle_finger",                                           include_bytes!("../../resources/emoji_icons/middle_finger.png")),
    "ðĪ" => ("money_mouth_face",                                        include_bytes!("../../resources/emoji_icons/money_face.png")),
    "ðĪ" => ("nerd, eyeglasses",                                        include_bytes!("../../resources/emoji_icons/nerd.png")),
    "ð" => ("neutral_face",                                            include_bytes!("../../resources/emoji_icons/neutral.png")),
    "ðĨģ" => ("party_face",                                              include_bytes!("../../resources/emoji_icons/party_face.png")),
    "ð" => ("party_popper",                                            include_bytes!("../../resources/emoji_icons/party_popper.png")),
    "ð" => ("pensive, sadder",                                         include_bytes!("../../resources/emoji_icons/pensive.png")),
    "ðĪĶ" => ("person_facepalming",                                      include_bytes!("../../resources/emoji_icons/facepalming.png")),
    "ð" => ("person_raising_hand, greeting",                           include_bytes!("../../resources/emoji_icons/raising_hand.png")),
    "ð" => ("person_gesturing_no",                                     include_bytes!("../../resources/emoji_icons/person_no.png")),
    "ð" => ("person_gesturing_ok, yes",                                include_bytes!("../../resources/emoji_icons/person_yes.png")),
    "ð" => ("person_bowing, pray, sorry",                              include_bytes!("../../resources/emoji_icons/bowing.png")),
    "ðŦ" => ("pistol, gun",                                             include_bytes!("../../resources/emoji_icons/pistol.png")),
    "ð" => ("pizza",                                                   include_bytes!("../../resources/emoji_icons/pizza.png")),
    "ðĨš" => ("pleading, big eyes, cute",                                include_bytes!("../../resources/emoji_icons/pleading.png")),
    "ðļ" => ("princess",                                                include_bytes!("../../resources/emoji_icons/princess.png")),
    "âš" => ("! relaxed",                                                include_bytes!("../../resources/emoji_icons/relaxed.png")),
    "ð" => ("relieved",                                                include_bytes!("../../resources/emoji_icons/relieved.png")),
    "ðĪ" => ("robot",                                                   include_bytes!("../../resources/emoji_icons/robot.png")),
    "ðĪĢ" => ("rolling_on_the_floor, rotfl",                             include_bytes!("../../resources/emoji_icons/rotfl.png")),
    "ð" => ("slightly_frowning, sad",                                  include_bytes!("../../resources/emoji_icons/frowning.png")),
    "ðą" => ("screaming",                                               include_bytes!("../../resources/emoji_icons/screaming.png")),
    "ð" => ("see_no_evil, monkey",                                     include_bytes!("../../resources/emoji_icons/see_no_evil.png")),
    "ðĐ" => ("shit",                                                    include_bytes!("../../resources/emoji_icons/shit.png")),
    "ðĪ·" => ("shrug",                                                   include_bytes!("../../resources/emoji_icons/shrugging.png")),
    "ðĪ" => ("face_with_bandage, sickness, operation, surgery, injury", include_bytes!("../../resources/emoji_icons/bandaged.png")),
    "ðĪ" => ("sign_of_the_horns, metal",                                include_bytes!("../../resources/emoji_icons/horns.png")),
    "â·ïļ" => ("skier",                                                   include_bytes!("../../resources/emoji_icons/skier.png")),
    "ð" => ("skull, dead",                                             include_bytes!("../../resources/emoji_icons/skull.png")),
    "ðī" => ("sleepy",                                                  include_bytes!("../../resources/emoji_icons/sleepy.png")),
    "ð" => ("slightly_smiling_face",                                   include_bytes!("../../resources/emoji_icons/smiling_slightly.png")),
    "ð" => ("smiling",                                                 include_bytes!("../../resources/emoji_icons/smiling.png")),
    "ð" => ("smiling_face_with_halo",                                  include_bytes!("../../resources/emoji_icons/halo.png")),
    "ðĨē" => ("smiling_face_with_tear",                                  include_bytes!("../../resources/emoji_icons/smiling_tear.png")),
    "ð" => ("smirk",                                                   include_bytes!("../../resources/emoji_icons/smirking.png")),
    "ðĪ§" => ("sneezing_face",                                           include_bytes!("../../resources/emoji_icons/sneezing.png")),
    "ðĶļââïļ" => ("superhero",                                               include_bytes!("../../resources/emoji_icons/superhero.png")),
    "ðĪĐ" => ("star_struck, starry eyes",                                include_bytes!("../../resources/emoji_icons/starstruck.png")),
    "ðĄïļ" => ("thermometer",                                             include_bytes!("../../resources/emoji_icons/thermometer.png")),
    "ðĪ" => ("thinking",                                                include_bytes!("../../resources/emoji_icons/thinking.png")),
    "ðĪŪ" => ("throwing_up",                                             include_bytes!("../../resources/emoji_icons/vomiting.png")),
    "ðđ" => ("tropical_drink",                                          include_bytes!("../../resources/emoji_icons/tropical_drink.png")),
    "ð" => ("unamused",                                                include_bytes!("../../resources/emoji_icons/unamused.png")),
    "ð" => ("upside-down",                                             include_bytes!("../../resources/emoji_icons/upside_down.png")),
    "ð" => ("waving_hand",                                             include_bytes!("../../resources/emoji_icons/waving_hand.png")),
    "ðķ" => ("without mouth, speechless, mute",                         include_bytes!("../../resources/emoji_icons/mouthless.png")),
    "ð" => ("wink",                                                    include_bytes!("../../resources/emoji_icons/wink.png")),
    "ð" => ("winking_face_with_tongue",                                include_bytes!("../../resources/emoji_icons/wink_tongue.png")),
    "ðĐ" => ("woman",                                                   include_bytes!("../../resources/emoji_icons/woman.png")),
    "ðĨī" => ("woozy, zany",                                             include_bytes!("../../resources/emoji_icons/woozy.png")),
    "ð" => ("yum, food, hungry, slurp, full belly",                    include_bytes!("../../resources/emoji_icons/yummy.png")),
    "ðĪŠ" => ("zany_face, o_O",                                          include_bytes!("../../resources/emoji_icons/zany.png")),
    "ðĪ" => ("zipper_mouth",                                            include_bytes!("../../resources/emoji_icons/zipper_mouth.png")),
    "ð§" => ("zombie",                                                  include_bytes!("../../resources/emoji_icons/zombie.png")),
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

    fn search(&mut self, pattern: String, sender: Sender<MessageEvent>, search_id: u32) {
        let pattern = pattern.chars().skip(1).collect::<String>();

        if pattern.len() > 0 {
            let search_result = EMOJI_ICON_PATTERNS
                .into_iter()
                .filter_map(|(emoji, (patterns, image_bytes))| {
                    if patterns.contains(&pattern) {
                        let shared_image =
                            SharedImage::from_image(PngImage::from_data(image_bytes).unwrap());

                        Some(SearchResultEntry::new(
                            shared_image.ok(),
                            patterns.to_string(),
                            Some(emoji.to_string()),
                            search_id,
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            sender.send(UpdateList(search_result));
        }
    }

    fn execute(&self, emoji: String) {
        copy_to_clipboard(emoji);
        process::exit(0);
    }
}

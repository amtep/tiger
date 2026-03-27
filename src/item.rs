//! Giant enum for all the [`Item`] types in the game.

pub use tiger_tables::item::Item;

use crate::block::Block;
use crate::db::Db;
#[cfg(feature = "eu5")]
use crate::eu5::item::injectable_eu5;
#[cfg(doc)]
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::pdxfile::PdxEncoding;
use crate::report::{Confidence, Severity};
use crate::token::Token;
#[cfg(feature = "vic3")]
use crate::vic3::item::injectable_vic3;

pub trait ItemExt {
    fn confidence(self) -> Confidence;
    fn severity(self) -> Severity;
    #[cfg(any(feature = "vic3", feature = "eu5"))]
    fn injectable(self) -> bool;
}

impl ItemExt for Item {
    /// Confidence value to use when reporting that an item is missing.
    /// Should be `Strong` for most, `Weak` for items that aren't defined anywhere but just used (such as gfx flags).
    fn confidence(self) -> Confidence {
        match self {
            #[cfg(feature = "jomini")]
            Item::AccessoryTag => Confidence::Weak,

            // GuiType and GuiTemplate are here because referring to templates in other mods is a
            // common compatibility technique.
            Item::GuiType | Item::GuiTemplate | Item::Sound => Confidence::Weak,

            #[cfg(feature = "ck3")]
            Item::AccoladeCategory
            | Item::BuildingGfx
            | Item::ClothingGfx
            | Item::CoaGfx
            | Item::CultureParameter
            | Item::MemoryCategory
            | Item::UnitGfx => Confidence::Weak,

            #[cfg(feature = "ck3")]
            Item::SpecialBuilding => Confidence::Reasonable,

            _ => Confidence::Strong,
        }
    }

    /// Severity value to use when reporting that an item is missing.
    /// * `Error` - most things
    /// * `Warning` - things that only impact visuals or presentation
    /// * `Untidy` - things that don't matter much at all
    /// * `Fatal` - things that cause crashes if they're missing
    ///
    /// This is only one piece of the severity puzzle. It can also depend on the caller who's expecting the item to exist.
    /// That part isn't handled yet.
    fn severity(self) -> Severity {
        match self {
            // GuiType and GuiTemplate are here because referring to templates in other mods is a
            // common compatibility technique.
            Item::GuiType | Item::GuiTemplate => Severity::Untidy,

            Item::File | Item::Localization | Item::MapEnvironment => Severity::Warning,

            #[cfg(feature = "jomini")]
            Item::Accessory
            | Item::AccessoryTag
            | Item::AccessoryVariation
            | Item::AccessoryVariationLayout
            | Item::AccessoryVariationTextures
            | Item::Coa
            | Item::CoaColorList
            | Item::CoaColoredEmblemList
            | Item::CoaPatternList
            | Item::CoaTemplate
            | Item::CoaTemplateList
            | Item::CoaTexturedEmblemList
            | Item::CustomLocalization
            | Item::EffectLocalization
            | Item::Ethnicity
            | Item::GameConcept
            | Item::NamedColor
            | Item::PortraitAnimation
            | Item::PortraitCamera
            | Item::PortraitEnvironment
            | Item::Sound
            | Item::TextFormat
            | Item::TextIcon
            | Item::TextureFile
            | Item::TriggerLocalization => Severity::Warning,

            #[cfg(feature = "ck3")]
            Item::AccoladeIcon
            | Item::ArtifactVisual
            | Item::BuildingGfx
            | Item::ClothingGfx
            | Item::CoaDynamicDefinition
            | Item::CoaGfx
            | Item::CultureAesthetic
            | Item::CultureCreationName
            | Item::EventBackground
            | Item::EventTheme
            | Item::EventTransition
            | Item::Flavorization
            | Item::GraphicalFaith
            | Item::ModifierFormat
            | Item::MottoInsert
            | Item::Motto
            | Item::Music
            | Item::Nickname
            | Item::RulerObjectiveType
            | Item::ScriptedIllustration
            | Item::UnitGfx => Severity::Warning,

            #[cfg(feature = "vic3")]
            Item::MapLayer
            | Item::ModifierTypeDefinition
            | Item::TerrainManipulator
            | Item::TerrainMask
            | Item::TerrainMaterial => Severity::Warning,

            #[cfg(feature = "hoi4")]
            Item::Sprite => Severity::Warning,

            _ => Severity::Error,
        }
    }

    #[cfg(any(feature = "vic3", feature = "eu5"))]
    fn injectable(self) -> bool {
        match Game::game() {
            #[cfg(feature = "vic3")]
            Game::Vic3 => injectable_vic3(self),
            #[cfg(feature = "eu5")]
            Game::Eu5 => injectable_eu5(self),
        }
    }
}

/// The callback type for adding one item instance to the database.
pub(crate) type ItemAdder = fn(&mut Db, Token, Block);

/// The specification for loading an [`Item`] type into the [`Db`].
///
/// An instance of this can be placed in every `data` module using the `inventory::submit!` macro.
/// This will register the loader so that the [`Everything`] object can load all defined items.
// Note that this is an enum so that users can more conveniently construct it. It used to be a
// struct with various constructor functions, but that didn't work because the ItemAdder type has a
// &mut in it, and that wasn't allowed in const functions even though the function pointer itself
// is const. See https://github.com/rust-lang/rust/issues/57349 for details.
// TODO: once that issue stabilizes, we can revisit the ItemLoader type.
pub(crate) enum ItemLoader {
    /// A convenience variant for loaders that are the most common type.
    ///
    /// * [`GameFlags`] is which games this item loader is for.
    /// * [`Item`] is the item type being loaded.
    ///
    /// The [`ItemAdder`] function does not have to load exclusively this type of item.
    /// Related items are ok. The main use of the [`Item`] field is to get the path for this item
    /// type, so that files are loaded from that folder.
    ///
    /// `Normal` loaders have extension `.txt`, `LoadAsFile::No`, and `Recursive::Maybe`. They default
    /// to a [`PdxEncoding`] appropriate to the game being validated.
    Normal(GameFlags, Item, ItemAdder),
    /// A variant that allows the full range of item loader behvavior.
    /// * [`PdxEncoding`] indicates whether to expect utf-8 and/or a BOM in the files.
    /// * The `&'static str` is the file extension to look for (including the dot).
    /// * [`LoadAsFile`] is whether to load the whole file as one item, or treat it as normal with a
    ///   series of items in one file.
    /// * [`Recursive`] indicates whether to load subfolders of the item's main folder.
    ///   `Recursive::Maybe` means apply game-dependent logic.
    Full(GameFlags, Item, PdxEncoding, &'static str, LoadAsFile, Recursive, ItemAdder),
}

inventory::collect!(ItemLoader);

impl ItemLoader {
    pub fn for_game(&self, game: Game) -> bool {
        let game_flags = match self {
            ItemLoader::Normal(game_flags, _, _)
            | ItemLoader::Full(game_flags, _, _, _, _, _, _) => game_flags,
        };
        game_flags.contains(GameFlags::from(game))
    }

    pub fn itype(&self) -> Item {
        match self {
            ItemLoader::Normal(_, itype, _) | ItemLoader::Full(_, itype, _, _, _, _, _) => *itype,
        }
    }

    pub fn encoding(&self) -> PdxEncoding {
        match self {
            ItemLoader::Normal(_, _, _) => {
                #[cfg(feature = "hoi4")]
                if Game::is_hoi4() {
                    return PdxEncoding::Utf8NoBom;
                }
                PdxEncoding::Utf8Bom
            }
            ItemLoader::Full(_, _, encoding, _, _, _, _) => *encoding,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ItemLoader::Normal(_, _, _) => ".txt",
            ItemLoader::Full(_, _, _, extension, _, _, _) => extension,
        }
    }

    pub fn whole_file(&self) -> bool {
        match self {
            ItemLoader::Normal(_, _, _) => false,
            ItemLoader::Full(_, _, _, _, load_as_file, _, _) => {
                matches!(load_as_file, LoadAsFile::Yes)
            }
        }
    }

    pub fn recursive(&self) -> bool {
        match self {
            ItemLoader::Normal(_, _, _) => {
                Game::is_ck3() && self.itype().path().starts_with("common/")
            }
            ItemLoader::Full(_, _, _, _, _, recursive, _) => match recursive {
                Recursive::Yes => true,
                Recursive::No => false,
                Recursive::Maybe => Game::is_ck3() && self.itype().path().starts_with("common/"),
            },
        }
    }

    pub fn adder(&self) -> ItemAdder {
        match self {
            ItemLoader::Normal(_, _, adder) | ItemLoader::Full(_, _, _, _, _, _, adder) => *adder,
        }
    }
}

pub enum LoadAsFile {
    Yes,
    No,
}

pub enum Recursive {
    Yes,
    No,
    #[allow(dead_code)]
    Maybe,
}

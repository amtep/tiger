//! The core [`Scopes`] type which tracks our knowledge about the types of in-game values.

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::game::Game;
use crate::helpers::{camel_case_to_separated_words, display_choices, snake_case_to_camel_case};
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{ErrorKey, err};
use crate::token::Token;

/// vic3 and ck3 need more than 64 bits, but the others don't.
#[cfg(any(feature = "vic3", feature = "ck3"))]
type ScopesBits = u128;
#[cfg(not(any(feature = "vic3", feature = "ck3")))]
type ScopesBits = u64;

bitflags! {
    /// This type represents our knowledge about the set of scope types that a script value can
    /// have. In most cases it's narrowed down to a single scope type, but not always.
    ///
    /// The available scope types depend on the game.
    /// They are listed in `event_scopes.log` from the game data dumps.
    // LAST UPDATED CK3 VERSION 1.16.0
    // LAST UPDATED VIC3 VERSION 1.8.1
    // LAST UPDATED IR VERSION 2.0.4
    //
    // Each scope type gets one bitflag. In order to keep the bit count down, scope types from
    // the different games have overlapping bitflags. Therefore, scope types from different games
    // should be kept carefully separated.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    #[rustfmt::skip] // having the cfg and the flag on one line is much more readable
    pub struct Scopes: ScopesBits {
        // Generic scope types
        const None = 1<<0;
        const Value = 1<<1;
        const Bool = 1<<2;
        const Flag = 1<<3;

        // Scope types shared by multiple games

        #[cfg(any(feature = "vic3", feature = "imperator", feature = "eu5"))]
        const Color = 1<<4;
        #[cfg(any(feature = "vic3", feature = "imperator", feature = "eu5", feature = "hoi4"))]
        const Country = 1<<5;
        const Character = 1<<6;
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "eu5", feature = "imperator"))]
        const Culture = 1<<7;
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "imperator", feature = "eu5"))]
        const Province = 1<<8;
        #[cfg(any(feature = "vic3", feature = "imperator", feature = "eu5"))]
        const Pop = 1<<9;
        #[cfg(any(feature = "vic3", feature = "imperator"))]
        const Party = 1<<10;
        #[cfg(feature = "eu5")]
        const PopType = 1<<10; // overlap with Party to save a bit
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "imperator", feature = "eu5"))]
        const Religion = 1<<11;
        #[cfg(any(feature = "vic3", feature = "imperator", feature = "hoi4"))]
        const State = 1<<12;
        #[cfg(feature = "eu5")]
        const Trait = 1<<12; // overlap with State to save a bit
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "imperator", feature = "eu5"))]
        const War = 1<<13;
        #[cfg(any(feature = "vic3", feature = "hoi4"))]
        const StrategicRegion = 1<<14;
        #[cfg(feature = "eu5")]
        const Invalid = 1<<14; // overlap with StrategicRegion to save a bit
        #[cfg(any(feature = "ck3", feature = "vic3"))]
        const Decision = 1<<15;
        #[cfg(feature = "eu5")]
        const Date = 1<<15; // overlap with Decision to save a bit

        // Scope types for CK3
        #[cfg(feature = "ck3")] const Accolade = 1<<16;
        #[cfg(feature = "ck3")] const AccoladeType = 1<<17;
        #[cfg(feature = "ck3")] const Activity = 1<<18;
        #[cfg(feature = "ck3")] const ActivityType = 1<<19;
        #[cfg(feature = "ck3")] const Army = 1<<20;
        #[cfg(feature = "ck3")] const Artifact = 1<<21;
        #[cfg(feature = "ck3")] const CasusBelli = 1<<22;
        #[cfg(feature = "ck3")] const CharacterMemory = 1<<23;
        #[cfg(feature = "ck3")] const Combat = 1<<24;
        #[cfg(feature = "ck3")] const CombatSide = 1<<25;
        #[cfg(feature = "ck3")] const CouncilTask = 1<<26;
        #[cfg(feature = "ck3")] const CulturePillar = 1<<27;
        #[cfg(feature = "ck3")] const CultureTradition = 1<<28;
        #[cfg(feature = "ck3")] const Doctrine = 1<<29;
        #[cfg(feature = "ck3")] const Dynasty = 1<<30;
        #[cfg(feature = "ck3")] const DynastyHouse = 1<<31;
        #[cfg(feature = "ck3")] const Faction = 1<<32;
        #[cfg(feature = "ck3")] const Faith = 1<<33;
        #[cfg(feature = "ck3")] const GovernmentType = 1<<34;
        #[cfg(feature = "ck3")] const GreatHolyWar = 1<<35;
        #[cfg(feature = "ck3")] const HolyOrder = 1<<36;
        #[cfg(feature = "ck3")] const Inspiration = 1<<37;
        #[cfg(feature = "ck3")] const LandedTitle = 1<<38;
        #[cfg(feature = "ck3")] const MercenaryCompany = 1<<39;
        #[cfg(feature = "ck3")] const Scheme = 1<<40;
        #[cfg(feature = "ck3")] const Secret = 1<<41;
        #[cfg(feature = "ck3")] const StoryCycle = 1<<42;
        #[cfg(feature = "ck3")] const Struggle = 1<<43;
        #[cfg(feature = "ck3")] const TitleAndVassalChange = 1<<44;
        #[cfg(feature = "ck3")] const Trait = 1<<45;
        #[cfg(feature = "ck3")] const TravelPlan = 1<<46;
        #[cfg(feature = "ck3")] const VassalContract = 1<<47;
        #[cfg(feature = "ck3")] const VassalObligationLevel = 1<<48;
        // CK3 1.11
        #[cfg(feature = "ck3")] const HoldingType = 1<<49;
        #[cfg(feature = "ck3")] const TaxSlot = 1<<50;
        // CK3 1.12
        #[cfg(feature = "ck3")] const EpidemicType = 1<<51;
        #[cfg(feature = "ck3")] const Epidemic = 1<<52;
        #[cfg(feature = "ck3")] const LegendType = 1<<53;
        #[cfg(feature = "ck3")] const Legend = 1<<54;
        #[cfg(feature = "ck3")] const GeographicalRegion = 1<<55;
        // CK3 1.13
        #[cfg(feature = "ck3")] const Domicile = 1<<56;
        #[cfg(feature = "ck3")] const AgentSlot = 1<<57;
        #[cfg(feature = "ck3")] const TaskContract = 1<<58;
        #[cfg(feature = "ck3")] const TaskContractType = 1<<59;
        #[cfg(feature = "ck3")] const Regiment = 1<<60;
        #[cfg(feature = "ck3")] const CasusBelliType = 1<<61;
        // CK3 1.15
        #[cfg(feature = "ck3")] const CourtPosition = 1<<62;
        #[cfg(feature = "ck3")] const CourtPositionType = 1<<63;
        // CK3 1.16
        #[cfg(feature = "ck3")] const Situation = 1<<64;
        #[cfg(feature = "ck3")] const SituationParticipantGroup = 1<<65;
        #[cfg(feature = "ck3")] const SituationSubRegion = 1<<66;
        #[cfg(feature = "ck3")] const Confederation = 1<<67;
        // CK3 1.18
        #[cfg(feature = "ck3")] const HouseAspiration = 1<<68;
        #[cfg(feature = "ck3")] const HouseRelation = 1<<69;
        #[cfg(feature = "ck3")] const HouseRelationType = 1<<70;
        #[cfg(feature = "ck3")] const HouseRelationLevel = 1<<71;
        #[cfg(feature = "ck3")] const ConfederationType = 1<<72;
        #[cfg(feature = "ck3")] const GreatProject = 1<<73;
        #[cfg(feature = "ck3")] const ProjectContribution = 1<<74;
        #[cfg(feature = "ck3")] const CultureInnovation = 1<<75;
        #[cfg(feature = "ck3")] const GreatProjectType = 1<<76;

        #[cfg(feature = "vic3")] const Battle = 1<<16;
        #[cfg(feature = "vic3")] const BattleSide = 1<<17;
        #[cfg(feature = "vic3")] const Building = 1<<18;
        #[cfg(feature = "vic3")] const BuildingType = 1<<19;
        #[cfg(feature = "vic3")] const CanalType = 1<<20;
        #[cfg(feature = "vic3")] const CivilWar = 1<<21;
        #[cfg(feature = "vic3")] const CulturalCommunity = 1<<22;
        #[cfg(feature = "vic3")] const NewCombatUnit = 1<<23;
        #[cfg(feature = "vic3")] const CommanderOrderType = 1<<24;
        #[cfg(feature = "vic3")] const CountryCreation = 1<<25;
        #[cfg(feature = "vic3")] const CountryDefinition = 1<<26;
        #[cfg(feature = "vic3")] const CountryFormation = 1<<27;
        #[cfg(feature = "vic3")] const Decree = 1<<28;
        #[cfg(feature = "vic3")] const DiplomaticAction = 1<<29;
        #[cfg(feature = "vic3")] const DiplomaticPact = 1<<30;
        #[cfg(feature = "vic3")] const DiplomaticPlay = 1<<31;
        #[cfg(feature = "vic3")] const DiplomaticRelations = 1<<32;
        #[cfg(feature = "vic3")] const Front = 1<<33;
        #[cfg(feature = "vic3")] const Goods = 1<<34;
        #[cfg(feature = "vic3")] const Hq = 1<<35;
        #[cfg(feature = "vic3")] const Ideology = 1<<36;
        #[cfg(feature = "vic3")] const Institution = 1<<37;
        #[cfg(feature = "vic3")] const InstitutionType = 1<<38;
        #[cfg(feature = "vic3")] const InterestMarker = 1<<39;
        #[cfg(feature = "vic3")] const InterestGroup = 1<<40;
        #[cfg(feature = "vic3")] const InterestGroupTrait = 1<<41;
        #[cfg(feature = "vic3")] const InterestGroupType = 1<<42;
        #[cfg(feature = "vic3")] const JournalEntry = 1<<43;
        #[cfg(feature = "vic3")] const Law = 1<<44;
        #[cfg(feature = "vic3")] const LawType = 1<<45;
        #[cfg(feature = "vic3")] const Market = 1<<46;
        #[cfg(feature = "vic3")] const MarketGoods = 1<<47;
        #[cfg(feature = "vic3")] const Objective = 1<<48;
        #[cfg(feature = "vic3")] const PoliticalMovement = 1<<49;
        #[cfg(feature = "vic3")] const PopType = 1<<50;
        #[cfg(feature = "vic3")] const ShippingLanes = 1<<51;
        #[cfg(feature = "vic3")] const StateRegion = 1<<52;
        #[cfg(feature = "vic3")] const StateTrait = 1<<53;
        #[cfg(feature = "vic3")] const Technology = 1<<54;
        #[cfg(feature = "vic3")] const TechnologyStatus = 1<<55;
        #[cfg(feature = "vic3")] const Theater = 1<<56;
        #[cfg(feature = "vic3")] const CombatUnitType = 1<<57;
        #[cfg(feature = "vic3")] const MilitaryFormation = 1<<58;
        #[cfg(feature = "vic3")] const Sway = 1<<59;
        #[cfg(feature = "vic3")] const StateGoods = 1<<60;
        #[cfg(feature = "vic3")] const DiplomaticDemand = 1<<61;
        #[cfg(feature = "vic3")] const Company = 1<<62;
        #[cfg(feature = "vic3")] const CompanyType = 1<<63;
        #[cfg(feature = "vic3")] const TravelNode = 1<<64;
        #[cfg(feature = "vic3")] const TravelNodeDefinition = 1<<65;
        #[cfg(feature = "vic3")] const TravelConnection = 1<<66;
        #[cfg(feature = "vic3")] const TravelConnectionDefinition = 1<<67;
        #[cfg(feature = "vic3")] const Invasion = 1<<68;
        #[cfg(feature = "vic3")] const MobilizationOption = 1<<69;
        #[cfg(feature = "vic3")] const PowerBlocPrincipleGroup = 1<<70;
        #[cfg(feature = "vic3")] const DiplomaticPlayType = 1<<71;
        #[cfg(feature = "vic3")] const DiplomaticCatalyst = 1<<72;
        #[cfg(feature = "vic3")] const DiplomaticCatalystType = 1<<73;
        #[cfg(feature = "vic3")] const DiplomaticCatalystCategory = 1<<74;
        #[cfg(feature = "vic3")] const PoliticalLobby = 1<<75;
        #[cfg(feature = "vic3")] const PoliticalLobbyType = 1<<76;
        #[cfg(feature = "vic3")] const PoliticalLobbyAppeasement = 1<<77;
        #[cfg(feature = "vic3")] const PowerBloc = 1<<78;
        #[cfg(feature = "vic3")] const PowerBlocIdentity = 1<<79;
        #[cfg(feature = "vic3")] const PowerBlocPrinciple = 1<<80;
        #[cfg(feature = "vic3")] const HarvestCondition = 1<<81;
        #[cfg(feature = "vic3")] const PoliticalMovementType = 1<<82;
        #[cfg(feature = "vic3")] const HarvestConditionType = 1<<83;
        #[cfg(feature = "vic3")] const TreatyArticle = 1<<84;
        #[cfg(feature = "vic3")] const TreatyOptions = 1<<85;
        #[cfg(feature = "vic3")] const TreatyArticleOptions = 1<<86;
        #[cfg(feature = "vic3")] const Treaty = 1<<87;
        #[cfg(feature = "vic3")] const BuildingGroup = 1<<88;
        #[cfg(feature = "vic3")] const Amendment = 1<<89;
        #[cfg(feature = "vic3")] const AmendmentType = 1<<90;
        #[cfg(feature = "vic3")] const GeographicRegion = 1<<91;
        #[cfg(feature = "vic3")] const WarGoal = 1<<92;
        #[cfg(feature = "vic3")] const WarGoalType = 1<<93;

        #[cfg(feature = "imperator")] const Area = 1<<16;
        #[cfg(feature = "imperator")] const CountryCulture = 1<<17;
        #[cfg(feature = "imperator")] const CultureGroup = 1<<18;
        #[cfg(feature = "imperator")] const Deity = 1<<19;
        #[cfg(feature = "imperator")] const Family = 1<<20;
        #[cfg(feature = "imperator")] const Governorship = 1<<21;
        #[cfg(feature = "imperator")] const GreatWork = 1<<22;
        #[cfg(feature = "imperator")] const Job = 1<<23;
        #[cfg(feature = "imperator")] const Legion = 1<<24;
        #[cfg(feature = "imperator")] const LevyTemplate = 1<<25;
        #[cfg(feature = "imperator")] const Region = 1<<26;
        #[cfg(feature = "imperator")] const Siege = 1<<27;
        #[cfg(feature = "imperator")] const SubUnit = 1<<28;
        #[cfg(feature = "imperator")] const Treasure = 1<<29;
        #[cfg(feature = "imperator")] const Unit = 1<<30;

        #[cfg(feature = "eu5")] const Location = 1<<16;
        #[cfg(feature = "eu5")] const Unit = 1<<17;
        #[cfg(feature = "eu5")] const SubUnit = 1<<18;
        #[cfg(feature = "eu5")] const Dynasty = 1<<19;
        #[cfg(feature = "eu5")] const War = 1<<20;
        #[cfg(feature = "eu5")] const Combat = 1<<21;
        #[cfg(feature = "eu5")] const CombatSide = 1<<22;
        #[cfg(feature = "eu5")] const Siege = 1<<23;
        #[cfg(feature = "eu5")] const ColonialCharter = 1<<24;
        #[cfg(feature = "eu5")] const Market = 1<<25;
        #[cfg(feature = "eu5")] const ProvinceDefinition = 1<<26;
        #[cfg(feature = "eu5")] const Area = 1<<27;
        #[cfg(feature = "eu5")] const Region = 1<<28;
        #[cfg(feature = "eu5")] const SubContinent = 1<<29;
        #[cfg(feature = "eu5")] const Continent = 1<<30;
        #[cfg(feature = "eu5")] const Group = 1<<31;
        #[cfg(feature = "eu5")] const Language = 1<<32;
        #[cfg(feature = "eu5")] const Rebels = 1<<33;
        #[cfg(feature = "eu5")] const Trade = 1<<34;
        #[cfg(feature = "eu5")] const ReligiousSchool = 1<<35;
        #[cfg(feature = "eu5")] const Goods = 1<<36;
        #[cfg(feature = "eu5")] const Demand = 1<<37;
        #[cfg(feature = "eu5")] const Privateer = 1<<38;
        #[cfg(feature = "eu5")] const Exploration = 1<<39;
        #[cfg(feature = "eu5")] const Mercenary = 1<<40;
        #[cfg(feature = "eu5")] const WorkOfArt = 1<<41;
        #[cfg(feature = "eu5")] const Government = 1<<42;
        #[cfg(feature = "eu5")] const InternationalOrganization = 1<<43;
        #[cfg(feature = "eu5")] const HolySite = 1<<44;
        #[cfg(feature = "eu5")] const Institution = 1<<45;
        #[cfg(feature = "eu5")] const Loan = 1<<46;
        #[cfg(feature = "eu5")] const Building = 1<<47;
        #[cfg(feature = "eu5")] const Law = 1<<48;
        #[cfg(feature = "eu5")] const Policy = 1<<49;
        #[cfg(feature = "eu5")] const Price = 1<<50;
        #[cfg(feature = "eu5")] const Situation = 1<<51;
        #[cfg(feature = "eu5")] const BuildingType = 1<<52;
        #[cfg(feature = "eu5")] const Disaster = 1<<53;
        #[cfg(feature = "eu5")] const ReligiousAspect = 1<<54;
        #[cfg(feature = "eu5")] const EstatePrivilege = 1<<55;
        #[cfg(feature = "eu5")] const CabinetAction = 1<<56;
        #[cfg(feature = "eu5")] const GovernmentReform = 1<<57;
        #[cfg(feature = "eu5")] const Cabinet = 1<<58;
        #[cfg(feature = "eu5")] const ProductionMethod = 1<<59;
        #[cfg(feature = "eu5")] const GraphicalCulture = 1<<60;
        #[cfg(feature = "eu5")] const DiseaseOutbreak = 1<<61;
        #[cfg(feature = "eu5")] const Disease = 1<<62;
        #[cfg(feature = "eu5")] const ParliamentIssue = 1<<63;
        #[cfg(feature = "eu5")] const ParliamentType = 1<<64;
        #[cfg(feature = "eu5")] const Resolution = 1<<65;
        #[cfg(feature = "eu5")] const God = 1<<66;
        #[cfg(feature = "eu5")] const Avatar = 1<<67;
        #[cfg(feature = "eu5")] const ReligiousFaction = 1<<68;
        #[cfg(feature = "eu5")] const SubjectType = 1<<69;
        #[cfg(feature = "eu5")] const Cardinal = 1<<70;
        #[cfg(feature = "eu5")] const ActiveResolution = 1<<71;
        #[cfg(feature = "eu5")] const Estate = 1<<72;
        #[cfg(feature = "eu5")] const AudioCulture = 1<<73;
        #[cfg(feature = "eu5")] const AdvanceType = 1<<74;
        #[cfg(feature = "eu5")] const CharacterInteraction = 1<<75;
        #[cfg(feature = "eu5")] const CountryInteraction = 1<<76;
        #[cfg(feature = "eu5")] const GenericAction = 1<<77;
        #[cfg(feature = "eu5")] const UnitType = 1<<78;
        #[cfg(feature = "eu5")] const LevySetup = 1<<79;
        #[cfg(feature = "eu5")] const ParliamentAgenda = 1<<80;
        #[cfg(feature = "eu5")] const CasusBelli = 1<<81;
        #[cfg(feature = "eu5")] const RelationType = 1<<82;
        #[cfg(feature = "eu5")] const DisasterType = 1<<83;
        #[cfg(feature = "eu5")] const SubUnitCategory = 1<<84;
        #[cfg(feature = "eu5")] const PeaceTreaty = 1<<85;
        #[cfg(feature = "eu5")] const ArtistType = 1<<86;
        #[cfg(feature = "eu5")] const WorkOfArtType = 1<<87;
        #[cfg(feature = "eu5")] const ChildEducation = 1<<88;
        #[cfg(feature = "eu5")] const Mission = 1<<89;
        #[cfg(feature = "eu5")] const MissionTask = 1<<90;
        #[cfg(feature = "eu5")] const RecruitmentMethod = 1<<91;
        #[cfg(feature = "eu5")] const RegencyType = 1<<92;
        #[cfg(feature = "eu5")] const UnitAbility = 1<<93;
        #[cfg(feature = "eu5")] const SocietalValueType = 1<<94;
        #[cfg(feature = "eu5")] const RoadType = 1<<95;
        #[cfg(feature = "eu5")] const LanguageFamily = 1<<96;
        #[cfg(feature = "eu5")] const CultureGroup = 1<<97;
        #[cfg(feature = "eu5")] const HeirSelection = 1<<98;
        #[cfg(feature = "eu5")] const EstateType = 1<<99;
        #[cfg(feature = "eu5")] const Dialect = 1<<100;
        #[cfg(feature = "eu5")] const Ethnicity = 1<<101;
        #[cfg(feature = "eu5")] const InternationalOrganizationType = 1<<102;
        #[cfg(feature = "eu5")] const Payment = 1<<103;
        #[cfg(feature = "eu5")] const SpecialStatus = 1<<104;
        #[cfg(feature = "eu5")] const LandOwnershipRule = 1<<105;
        #[cfg(feature = "eu5")] const WeatherSystem = 1<<106;
        #[cfg(feature = "eu5")] const FormableCountry = 1<<107;
        #[cfg(feature = "eu5")] const Hegemony = 1<<108;
        #[cfg(feature = "eu5")] const HolySiteDefinition = 1<<109;
        #[cfg(feature = "eu5")] const HolySiteType = 1<<110;
        #[cfg(feature = "eu5")] const CountryRank = 1<<111;
        #[cfg(feature = "eu5")] const LocationRank = 1<<112;
        #[cfg(feature = "eu5")] const ReligiousFocus = 1<<113;
        #[cfg(feature = "eu5")] const ReligiousFigure = 1<<114;
        #[cfg(feature = "eu5")] const Climate = 1<<115;
        #[cfg(feature = "eu5")] const Vegetation = 1<<116;
        #[cfg(feature = "eu5")] const Topography = 1<<117;
        #[cfg(feature = "eu5")] const Age = 1<<118;
        #[cfg(feature = "eu5")] const EmploymentSystem = 1<<119;
        #[cfg(feature = "eu5")] const MilitaryStance = 1<<120;
        #[cfg(feature = "eu5")] const UnitTemplate = 1<<121;
        #[cfg(feature = "eu5")] const UnitFormationPreference = 1<<122;
        #[cfg(feature = "eu5")] const ScriptableHintDefinition = 1<<123;
        #[cfg(feature = "eu5")] const ScriptedGeography = 1<<124;

        #[cfg(feature = "hoi4")] const Ace = 1<<16;
        #[cfg(feature = "hoi4")] const Combatant = 1<<17;
        #[cfg(feature = "hoi4")] const Division = 1<<18;
        #[cfg(feature = "hoi4")] const IndustrialOrg = 1<<19;
        #[cfg(feature = "hoi4")] const Operation = 1<<20;
        #[cfg(feature = "hoi4")] const PurchaseContract = 1<<21;
        #[cfg(feature = "hoi4")] const RaidInstance = 1<<22;
        #[cfg(feature = "hoi4")] const SpecialProject = 1<<23;
        // These two "combined" ones represent the odd scopes created for events.
        #[cfg(feature = "hoi4")] const CombinedCountryAndState = 1<<24;
        #[cfg(feature = "hoi4")] const CombinedCountryAndCharacter = 1<<25;
    }
}

// These have to be expressed a bit awkwardly because the binary operators are not `const`.
// TODO: Scopes::all() returns a too-large set if multiple features are enabled.
impl Scopes {
    pub const fn non_primitive() -> Scopes {
        Scopes::all()
            .difference(Scopes::None.union(Scopes::Value).union(Scopes::Bool).union(Scopes::Flag))
    }

    pub const fn primitive() -> Scopes {
        Scopes::Value.union(Scopes::Bool).union(Scopes::Flag)
    }

    pub const fn all_but_none() -> Scopes {
        Scopes::all().difference(Scopes::None)
    }

    /// Read a scope type in string form and return it as a [`Scopes`] value.
    pub fn from_snake_case(s: &str) -> Option<Scopes> {
        #[cfg(feature = "ck3")]
        if Game::is_ck3() {
            // Deal with some exceptions to the general pattern
            match s {
                "ghw" => return Some(Scopes::GreatHolyWar),
                "story" => return Some(Scopes::StoryCycle),
                "great_holy_war" | "story_cycle" => return None,
                _ => (),
            }
        }

        Scopes::from_name(&snake_case_to_camel_case(s))
    }

    /// Similar to `from_snake_case`, but allows multiple scopes separated by `|`
    /// Returns None if any of the conversions fail.
    pub fn from_snake_case_multi(s: &str) -> Option<Scopes> {
        let mut scopes = Scopes::empty();
        for part in s.split('|') {
            if let Some(scope) = Scopes::from_snake_case(part) {
                scopes |= scope;
            } else {
                return None;
            }
        }
        // If `scopes` is still empty then probably `s` was empty.
        // Remember that `Scopes::empty()` is different from a bitfield containing `Scopes::None`.
        if scopes == Scopes::empty() {
            return None;
        }
        Some(scopes)
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        if *self == Scopes::all() {
            write!(f, "any scope")
        } else if *self == Scopes::primitive() {
            write!(f, "any primitive scope")
        } else if *self == Scopes::non_primitive() {
            write!(f, "non-primitive scope")
        } else if *self == Scopes::all_but_none() {
            write!(f, "any except none scope")
        } else {
            let mut vec = Vec::new();
            for (name, _) in self.iter_names() {
                vec.push(camel_case_to_separated_words(name));
            }
            let vec: Vec<&str> = vec.iter().map(String::as_ref).collect();
            display_choices(f, &vec, "or")
        }
    }
}

/// A description of the constraints on a value with a prefix such as `var:` or `list_size:`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArgumentValue {
    /// The value must be an expression that resolves to a scope object of the given type.
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    Scope(Scopes),
    /// The value must be the name of an item of the given item type.
    Item(Item),
    /// The value can be either a Scope or an Item
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    ScopeOrItem(Scopes, Item),
    /// The value can be a trait name or `trait|track`.
    #[cfg(feature = "ck3")]
    TraitTrack,
    /// The value must be the name of a modif
    #[cfg(any(feature = "vic3", feature = "imperator"))]
    Modif,
    /// The value must be a single word
    #[cfg(any(feature = "vic3", feature = "ck3"))]
    Identifier(&'static str),
    /// The value can be anything
    UncheckedValue,
    /// This trigger no longer exists. Arguments are version and explanation
    #[cfg(feature = "ck3")]
    Removed(&'static str, &'static str),
}

/// Look up an "event link", which is a script token that looks up something related
/// to a scope value and returns another scope value.
///
/// `name` is the token. `inscopes` is the known scope context of this token.
/// `inscopes` is only used for some special-case event links whose output scope type
/// depends on their input scope type.
///
/// Returns a pair of `Scopes`. The first is the scope types this token can accept as input,
/// and the second is the scope types it may return.
#[allow(unused_variables)] // inscopes is only used for vic3
pub fn scope_to_scope(name: &Token, inscopes: Scopes) -> Option<(Scopes, Scopes)> {
    let scope_to_scope = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::targets::scope_to_scope,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::targets::scope_to_scope,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::targets::scope_to_scope,
        #[cfg(feature = "eu5")]
        Game::Eu5 => crate::eu5::tables::targets::scope_to_scope,
        #[cfg(feature = "hoi4")]
        Game::Hoi4 => crate::hoi4::tables::targets::scope_to_scope,
    };
    let scope_to_scope_removed = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::targets::scope_to_scope_removed,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::targets::scope_to_scope_removed,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::targets::scope_to_scope_removed,
        #[cfg(feature = "eu5")]
        Game::Eu5 => crate::eu5::tables::targets::scope_to_scope_removed,
        #[cfg(feature = "hoi4")]
        Game::Hoi4 => crate::hoi4::tables::targets::scope_to_scope_removed,
    };

    let name_lc = name.as_str().to_ascii_lowercase();
    #[allow(unused_assignments)] // `from` only used for vic3
    if let scopes @ Some((from, _)) = scope_to_scope(&name_lc) {
        #[cfg(feature = "vic3")]
        if Game::is_vic3() && name_lc == "type" {
            // Special case for "type" because it goes from specific scope types to specific
            // other scope types.
            let mut outscopes = Scopes::empty();
            if inscopes.contains(Scopes::Building) {
                outscopes |= Scopes::BuildingType;
            }
            if inscopes.contains(Scopes::Company) {
                outscopes |= Scopes::CompanyType;
            }
            if inscopes.contains(Scopes::DiplomaticPlay) {
                outscopes |= Scopes::DiplomaticPlayType;
            }
            if inscopes.contains(Scopes::DiplomaticCatalyst) {
                outscopes |= Scopes::DiplomaticCatalystType;
            }
            if inscopes.contains(Scopes::PoliticalLobby) {
                outscopes |= Scopes::PoliticalLobbyType;
            }
            if inscopes.contains(Scopes::Institution) {
                outscopes |= Scopes::InstitutionType;
            }
            if inscopes.contains(Scopes::InterestGroup) {
                outscopes |= Scopes::InterestGroupType;
            }
            if inscopes.contains(Scopes::Law) {
                outscopes |= Scopes::LawType;
            }
            if inscopes.contains(Scopes::PoliticalMovement) {
                outscopes |= Scopes::PoliticalMovementType;
            }
            if inscopes.contains(Scopes::HarvestCondition) {
                outscopes |= Scopes::HarvestConditionType;
            }
            if !outscopes.is_empty() {
                return Some((from, outscopes));
            }
        }
        scopes
    } else if let Some((version, explanation)) = scope_to_scope_removed(&name_lc) {
        let msg = format!("`{name}` was removed in {version}");
        err(ErrorKey::Removed).strong().msg(msg).info(explanation).loc(name).push();
        Some((Scopes::all(), Scopes::all_but_none()))
    } else {
        None
    }
}

/// Look up a prefixed token that is used to look up items in the game database.
///
/// For example, `character:alexander_the_great` to fetch that character as a scope value.
///
/// Some prefixes have an input scope, and they look up something related to the input scope value.
///
/// Returns a pair of `Scopes` and the type of argument it accepts.
/// The first `Scopes` is the scope types this token can accept as input, and the second one is
/// the scope types it may return. The first will be `Scopes::None` if it needs no input.
pub fn scope_prefix(prefix: &Token) -> Option<(Scopes, Scopes, ArgumentValue)> {
    let scope_prefix = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::targets::scope_prefix,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::targets::scope_prefix,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::targets::scope_prefix,
        #[cfg(feature = "eu5")]
        Game::Eu5 => crate::eu5::tables::targets::scope_prefix,
        #[cfg(feature = "hoi4")]
        Game::Hoi4 => crate::hoi4::tables::targets::scope_prefix,
    };
    let prefix_lc = prefix.as_str().to_ascii_lowercase();
    scope_prefix(&prefix_lc)
}

/// Look up a token that's an invalid target, and see if it might be missing a prefix.
/// Return the prefix if one was found.
///
/// `scopes` should be a singular `Scopes` flag.
///
/// Example: if the token is "irish" and `scopes` is `Scopes::Culture` then return
/// `Some("culture")` to indicate that the token should have been "culture:irish".
pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::scopes::needs_prefix(arg, data, scopes),
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::scopes::needs_prefix(arg, data, scopes),
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::scopes::needs_prefix(arg, data, scopes),
        #[cfg(feature = "eu5")]
        Game::Eu5 => crate::eu5::scopes::needs_prefix(arg, data, scopes),
        #[cfg(feature = "hoi4")]
        Game::Hoi4 => crate::hoi4::scopes::needs_prefix(arg, data, scopes),
    }
}

/// Look up an iterator, which is a script element that executes its block multiple times, once for
/// each applicable scope value. Iterators may be builtin (the usual case) or may be scripted lists.
///
/// `name` is the name of the iterator, without its `any_`, `every_`, `random_` or `ordered_` prefix.
/// `sc` is a [`ScopeContext`], only used for validating scripted lists.
///
/// Returns a pair of `Scopes`. The first is the scope types this token can accept as input,
/// and the second is the scope types it may return.
/// The first will be `Scopes::None` if it needs no input.
pub fn scope_iterator(
    name: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
) -> Option<(Scopes, Scopes)> {
    let scope_iterator = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::iterators::iterator,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::iterators::iterator,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::iterators::iterator,
        #[cfg(feature = "eu5")]
        Game::Eu5 => crate::eu5::tables::iterators::iterator,
        #[cfg(feature = "hoi4")]
        Game::Hoi4 => crate::hoi4::tables::iterators::iterator,
    };
    let scope_iterator_removed = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::iterators::iterator_removed,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::iterators::iterator_removed,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::iterators::iterator_removed,
        #[cfg(feature = "eu5")]
        Game::Eu5 => crate::eu5::tables::iterators::iterator_removed,
        #[cfg(feature = "hoi4")]
        Game::Hoi4 => crate::hoi4::tables::iterators::iterator_removed,
    };

    let name_lc = Lowercase::new(name.as_str());
    if let scopes @ Some(_) = scope_iterator(&name_lc, name, data) {
        return scopes;
    }
    if let Some((version, explanation)) = scope_iterator_removed(name_lc.as_str()) {
        let msg = format!("`{name}` iterators were removed in {version}");
        err(ErrorKey::Removed).strong().msg(msg).info(explanation).loc(name).push();
        return Some((Scopes::all(), Scopes::all()));
    }
    #[cfg(feature = "jomini")]
    if Game::is_jomini() && data.scripted_lists.exists(name.as_str()) {
        data.scripted_lists.validate_call(name, data, sc);
        return data
            .scripted_lists
            .base(name)
            .and_then(|base| scope_iterator(&Lowercase::new(base.as_str()), base, data));
    }
    #[cfg(feature = "hoi4")]
    let _ = &data; // mark parameter used
    #[cfg(feature = "hoi4")]
    let _ = &sc; // mark parameter used
    None
}

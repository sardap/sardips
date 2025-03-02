use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Duration;
use std::{i32, i64};

use crate::{money::Wallet, sardip_save::SardipLoadingState, simulation::SimulationUpdate};
use bevy::prelude::*;
use sardips_core::money_core::Money;
use sardips_core::rand_utils::{gen_f32_range, gen_f64_range, NewBuilder, WalkerTable};
use sardips_core::wrapped_vec::WrappingVec;
use serde::{Deserialize, Serialize};
use shared_deps::bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};
use shared_deps::moonshine_save::save::Save;

pub struct StockMarketPlugin;

impl Plugin for StockMarketPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Company>()
            .register_type::<StockMarketGhost>()
            .register_type::<SharePortfolio>()
            .register_type::<OrderBook>()
            .register_type::<QuarterManger>()
            .register_type::<BuySellOrchestrator>()
            .register_type::<StockMarketAI>()
            .register_type::<ShareHistory>()
            .add_systems(
                OnEnter(SardipLoadingState::Loaded),
                (
                    create_order_book.run_if(not(resource_exists::<OrderBook>)),
                    create_quarter_manager.run_if(not(resource_exists::<QuarterManger>)),
                    create_buy_sell_orchestrator
                        .run_if(not(resource_exists::<BuySellOrchestrator>)),
                    spawn_companies,
                    spawn_ghosts,
                ),
            )
            .add_systems(Update, (add_rng_to_stock_stuff, allocate_stocks))
            .add_systems(
                SimulationUpdate,
                (
                    tick_quarter,
                    update_company_price_cache,
                    generate_buy_sell_activity,
                    process_orders,
                )
                    .chain(),
            );
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct SharePortfolio {
    pub owned_shares: HashMap<Entity, u64>,
}

impl SharePortfolio {
    pub fn new_player_portfolio() -> Self {
        Self::default()
    }

    pub fn add_shares(&mut self, company: Entity, quantity: u64) {
        let shares = self.owned_shares.entry(company).or_insert(0);
        *shares += quantity;
    }

    pub fn remove_shares(&mut self, company: Entity, quantity: u64) {
        let shares = self.owned_shares.entry(company).or_insert(0);
        if *shares < quantity {
            *shares = 0;
        } else {
            *shares -= quantity;
        }
    }

    pub fn get_count(&self, company: &Entity) -> u64 {
        *self.owned_shares.get(company).unwrap_or(&0)
    }
}

#[derive(Default, Deserialize, Serialize, Clone, Copy, IntoStaticStr, Reflect)]
#[reflect_value(Deserialize, Serialize)]
pub enum Industry {
    #[default]
    Tech,
    Telecommunications,
    Finance,
    Manufacturing,
    Retail,
    Healthcare,
    Energy,
    RealEstate,
    Transportation,
    Food,
    Entertainment,
    Mining,
}

impl Industry {
    pub fn name_key(&self) -> String {
        let str: &'static str = self.into();
        format!("industry.{}.name", str.to_lowercase())
    }
}

#[derive(Default, Deserialize, Serialize, Clone, Reflect)]
#[reflect_value(Deserialize, Serialize)]
pub struct CompanyHistory {
    pub quarter: u32,
    pub assets: Money,
    pub revenue: Money,
    pub expenses: Money,
    pub total_shares: u64,
    pub dividend_paid: Money,
    pub performance: PerformanceRanking,
}

#[derive(Default, Deserialize, Serialize, Clone, Copy, Reflect)]
#[reflect_value(Deserialize, Serialize)]
pub enum PerformanceRanking {
    Extraordinary,
    Excellent,
    Good,
    #[default]
    Average,
    Poor,
    Terrible,
    Horrific,
}

const REDUCTION_RATE: f64 = 1.0;

impl PerformanceRanking {
    const TRANSITIONS: [PerformanceRanking; 7] = [
        PerformanceRanking::Extraordinary,
        PerformanceRanking::Excellent,
        PerformanceRanking::Good,
        PerformanceRanking::Average,
        PerformanceRanking::Poor,
        PerformanceRanking::Terrible,
        PerformanceRanking::Horrific,
    ];

    pub fn range(&self) -> std::ops::Range<f64> {
        let range = match self {
            Self::Extraordinary => 0.25..1.0,
            Self::Excellent => 0.1..0.3,
            Self::Good => 0.05..0.1,
            Self::Average => -0.03..0.025,
            Self::Poor => -0.2..-0.05,
            Self::Terrible => -0.15..-0.1,
            Self::Horrific => -1.0..-0.25,
        };

        (range.start * REDUCTION_RATE)..(range.end * REDUCTION_RATE)
    }

    pub fn asset_range(&self) -> std::ops::Range<f64> {
        let range = self.range();

        (range.start * REDUCTION_RATE * 0.25)..(range.end * REDUCTION_RATE * 0.25)
    }

    pub fn expense_range(&self) -> std::ops::Range<f64> {
        let range = match self {
            Self::Extraordinary => -0.1..0.05,
            Self::Excellent => -0.05..0.025,
            Self::Good => -0.025..0.01,
            Self::Average => -0.01..0.01,
            Self::Poor => 0.05..0.1,
            Self::Terrible => 0.025..0.05,
            Self::Horrific => 0.01..0.1,
        };

        (range.start * REDUCTION_RATE)..(range.end * REDUCTION_RATE)
    }

    pub fn next<T: DelegatedRng>(&self, rng: &mut T) -> Self {
        lazy_static! {
            static ref EXTRAORDINARY_TRANS: WalkerTable =
                WalkerTable::new(&[50, 100, 100, 250, 250, 200, 50]);
            static ref EXCELLENT_TRANS: WalkerTable =
                WalkerTable::new(&[20, 200, 350, 350, 50, 22, 8]);
            static ref GOOD_TRANS: WalkerTable = WalkerTable::new(&[10, 100, 500, 350, 30, 5, 5]);
            static ref AVERAGE_TRANS: WalkerTable = WalkerTable::new(&[1, 37, 100, 800, 50, 11, 1]);
            static ref POOR_TRANS: WalkerTable = WalkerTable::new(&[5, 5, 30, 350, 500, 100, 10]);
            static ref TERRIBLE_TRANS: WalkerTable =
                WalkerTable::new(&[8, 22, 50, 350, 350, 200, 20]);
            static ref HORRIFIC_TRANS: WalkerTable =
                WalkerTable::new(&[10, 20, 20, 100, 25, 25, 800]);
        }
        let table = match *self {
            PerformanceRanking::Extraordinary => &*EXTRAORDINARY_TRANS,
            PerformanceRanking::Excellent => &*EXCELLENT_TRANS,
            PerformanceRanking::Good => &*GOOD_TRANS,
            PerformanceRanking::Average => &*AVERAGE_TRANS,
            PerformanceRanking::Poor => &*POOR_TRANS,
            PerformanceRanking::Terrible => &*TERRIBLE_TRANS,
            PerformanceRanking::Horrific => &*HORRIFIC_TRANS,
        };

        PerformanceRanking::TRANSITIONS[table.next_rng(rng)]
    }
}

#[derive(Default, Component, Deserialize, Serialize, Clone, Reflect)]
#[reflect_value(Deserialize, Serialize, Component)]
pub struct ShareHistory {
    pub history: WrappingVec<BuyOrderBrief, 300>,
    pub cached_price: Money,
    pub dirty_price: bool,
}

impl ShareHistory {
    pub fn new(starting_price: Money) -> Self {
        let mut history = WrappingVec::new();
        history.push(BuyOrderBrief::new(1000, starting_price));

        Self {
            history,
            cached_price: starting_price,
            dirty_price: false,
        }
    }

    pub fn add_entry(&mut self, price: Money, volume: u64) {
        self.history.push(BuyOrderBrief::new(volume, price));
        self.dirty_price = true;
    }

    pub fn update_cached_price(&mut self) {
        if !self.dirty_price {
            return;
        }
        self.cached_price = self.price();
        self.dirty_price = false;
    }

    pub fn price(&self) -> Money {
        const N: u64 = 20000;
        let mut total_price = 0;
        let mut total_volume = 0;
        for entry in self.history.iter().rev() {
            let remaining_needed = N - total_volume;
            let taken_amount = entry.quantity.min(remaining_needed);
            total_price += entry.price * taken_amount as i64;
            total_volume += taken_amount;
            if taken_amount != entry.quantity {
                break;
            }
        }

        if total_volume == 0 || total_price == 0 {
            return 1;
        }

        (total_price as f64 / total_volume as f64) as Money
    }
}

#[derive(Default, Deserialize, Serialize, Clone, Reflect, PartialEq)]
#[reflect_value(Deserialize, Serialize)]
pub struct CompanyPerformance {
    pub pb_ratio: f32,
    pub pe_ratio: f32,
    pub peg_ratio: Option<f32>,
    pub dividend_yield: f32,
    pub stock_price: Money,
}

impl CompanyPerformance {
    pub fn new(company: &Company, share_history: &ShareHistory) -> Self {
        // Get entry that has no performance data
        let history = company.history.last().unwrap();

        let share_price = share_history.price();

        let pb_ratio: f32;
        {
            let market_value = (company.existing_shares as i64 * share_price) as f64 / 100.;
            let book_value = company.book_value() as f64 / 100.;
            pb_ratio = (market_value / book_value) as f32;
        }

        let pe_ratio: f32;
        {
            let stock_price = share_price;
            let earnings = history.revenue - history.expenses;
            let earning_per_share = earnings as f64 / company.existing_shares as f64;
            pe_ratio = (stock_price as f64 / earning_per_share) as f32;
        }

        let peg_ratio: Option<f32>;
        {
            if company.history.len() < 2 {
                peg_ratio = None;
            } else {
                let last_history = &company.history[company.history.len() - 2];
                let last_eps = (last_history.revenue - last_history.expenses) as f64
                    / company.existing_shares as f64;
                let this_eps =
                    (history.revenue - history.expenses) as f64 / company.existing_shares as f64;
                let eps_growth = (this_eps - last_eps) / last_eps;

                peg_ratio = Some(pe_ratio / eps_growth as f32);
            }
        }

        CompanyPerformance {
            pb_ratio,
            pe_ratio,
            peg_ratio,
            dividend_yield: 0.0,
            stock_price: share_price,
        }
    }
}

pub struct CompanyRank {
    pub pe_percentile: f32,
    pub pb_percentile: f32,
    pub peg_percentile: f32,
}

impl CompanyRank {
    pub fn new_ranking(companies: &[(Entity, CompanyPerformance)]) -> HashMap<Entity, Self> {
        let mut company_lookup = Vec::with_capacity(companies.len());
        for company in 0..companies.len() {
            company_lookup.push(company);
        }

        let mut pe_ranking = company_lookup.clone().into_iter().collect::<Vec<_>>();
        pe_ranking.sort_by(|a, b| {
            companies[*a]
                .1
                .pe_ratio
                .partial_cmp(&companies[*b].1.pe_ratio)
                .unwrap()
        });

        let mut pb_ranking = company_lookup.clone().into_iter().collect::<Vec<_>>();
        pb_ranking.sort_by(|a, b| {
            companies[*a]
                .1
                .pb_ratio
                .partial_cmp(&companies[*b].1.pb_ratio)
                .unwrap()
                .reverse()
        });

        let mut peg_ranking = company_lookup.clone().into_iter().collect::<Vec<_>>();
        peg_ranking.sort_by(|a, b| {
            companies[*a]
                .1
                .peg_ratio
                .partial_cmp(&companies[*b].1.peg_ratio)
                .unwrap()
        });

        let mut company_rankings = HashMap::new();
        for (i, company_index) in company_lookup.iter().enumerate() {
            let pe_rank = pe_ranking.iter().position(|x| *x == i).unwrap();
            let pb_rank = pb_ranking.iter().position(|x| *x == i).unwrap();
            let peg_rank = peg_ranking.iter().position(|x| *x == i).unwrap();
            company_rankings.insert(
                companies[*company_index].0,
                CompanyRank {
                    pe_percentile: (pe_rank + 1) as f32 / companies.len() as f32,
                    pb_percentile: (pb_rank + 1) as f32 / companies.len() as f32,
                    peg_percentile: (peg_rank + 1) as f32 / companies.len() as f32,
                },
            );
        }

        company_rankings
    }
}

#[derive(Default, Component, Deserialize, Serialize, Clone, Reflect)]
#[reflect_value(Component, Deserialize, Serialize)]
pub struct Company {
    pub ticker: String,
    pub existing_shares: u64,
    pub history: Vec<CompanyHistory>,
    pub performance_history: Vec<CompanyPerformance>,
    pub industries: Vec<(f32, Industry)>,
}

impl Company {
    pub fn book_value(&self) -> Money {
        self.history.last().unwrap().assets
    }

    pub fn name_key(&self) -> String {
        format!("company.{}.name", self.ticker.to_lowercase())
    }

    pub fn description_key(&self) -> String {
        format!("company.{}.description", self.ticker.to_lowercase())
    }

    pub fn market_value(&self, share_history: &ShareHistory) -> i128 {
        self.existing_shares as i128 * share_history.price() as i128
    }
}

#[derive(Bundle)]
pub struct CompanyBundle {
    pub company: Company,
    pub share_history: ShareHistory,
    pub wallet: Wallet,
    pub share_portfolio: SharePortfolio,
    pub rng: RngComponent,
    pub save: Save,
}

pub const PB_MAX_SCORE: f32 = 100.;
const PE_MAX_SCORE: f32 = 100.;
const PEG_MAX_SCORE: f32 = 100.;
const SCORE_PARTS: [f32; 3] = [PB_MAX_SCORE, PE_MAX_SCORE, PEG_MAX_SCORE];
lazy_static! {
    static ref SCORE_SUM: f32 = SCORE_PARTS.iter().sum();
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Reflect, PartialEq, Eq, PartialOrd, Ord)]
enum BuyThreshold {
    Never,
    Low,
    Medium,
    High,
    Must,
}

impl BuyThreshold {
    fn from_score(score: f32) -> Self {
        match score {
            s if s < 15.0 => Self::Never,
            s if s < 30.0 => Self::Low,
            s if s < 70.0 => Self::Medium,
            s if s < 90.0 => Self::High,
            _ => Self::Must,
        }
    }

    fn buy_prob(&self) -> f32 {
        match self {
            Self::Never => 0.1,
            Self::Low => 0.2,
            Self::Medium => 0.5,
            Self::High => 0.7,
            Self::Must => 0.9,
        }
    }

    fn price_modifier(&self) -> f32 {
        match self {
            Self::Never => 0.5,
            Self::Low => 0.9,
            Self::Medium => 1.0,
            Self::High => 1.1,
            Self::Must => 1.2,
        }
    }

    pub fn invert(&self) -> Self {
        match self {
            Self::Never => Self::Must,
            Self::Low => Self::High,
            Self::Medium => Self::Medium,
            Self::High => Self::Low,
            Self::Must => Self::Never,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, PartialOrd)]
#[reflect_value(Deserialize, Serialize, PartialEq)]
pub struct BuyOrder {
    pub lifetime: Duration,
    pub cycles: u16,
    pub company: Entity,
    pub quantity: u64,
    pub remaining_quantity: u64,
    pub price: Money,
    pub buyer: Entity,
}

impl BuyOrder {
    pub fn new(company: Entity, quantity: u64, price: Money, buyer: Entity) -> Self {
        Self {
            lifetime: Duration::ZERO,
            cycles: 0,
            company,
            quantity,
            remaining_quantity: quantity,
            price,
            buyer,
        }
    }
}

impl Ord for BuyOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.cmp(&other.price)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Reflect, PartialOrd)]
#[reflect_value(Deserialize, Serialize, PartialEq)]
pub struct SellOrder {
    pub lifetime: Duration,
    pub cycles: u64,
    pub company: Entity,
    pub quantity: u64,
    pub remaining_quantity: u64,
    pub price: Money,
    pub seller: Entity,
}

impl SellOrder {
    pub fn new(company: Entity, quantity: u64, price: Money, seller: Entity) -> Self {
        Self {
            lifetime: Duration::ZERO,
            cycles: 0,
            company,
            quantity,
            remaining_quantity: quantity,
            price,
            seller,
        }
    }
}
#[derive(Default, Reflect, Clone, Serialize, Deserialize)]
#[reflect_value(Deserialize, Serialize)]
pub struct BuyOrderBrief {
    pub quantity: u64,
    pub price: Money,
}

impl BuyOrderBrief {
    pub fn new(quantity: u64, price: Money) -> Self {
        Self { quantity, price }
    }
}

#[derive(Default, Reflect, Clone, Serialize, Deserialize, Resource)]
#[reflect_value(Deserialize, Serialize, Resource)]
pub struct OrderBook {
    pub buy_orders: Vec<BuyOrder>,
    pub sell_orders: HashMap<Entity, Vec<SellOrder>>,
}

impl OrderBook {
    pub fn add(&mut self, order: NewOrder) {
        match order {
            NewOrder::Buy(buy_order) => {
                self.buy_orders.push(buy_order);
            }
            NewOrder::Sell(new_sell_order) => {
                let sell_orders = self
                    .sell_orders
                    .entry(new_sell_order.company)
                    .or_insert(Vec::new());

                let index =
                    match sell_orders.binary_search_by(|x| x.price.cmp(&new_sell_order.price)) {
                        Ok(i) => i,
                        Err(i) => i,
                    };

                sell_orders.insert(index, new_sell_order);
            }
        }
    }

    pub fn get_sell_orders<T: Into<Entity>>(&self, company: T) -> &Vec<SellOrder> {
        lazy_static! {
            static ref EMPTY: Vec<SellOrder> = Vec::new();
        }

        let company = company.into();
        self.sell_orders
            .get(&company)
            .map_or(&(*EMPTY), |orders| orders)
    }
}

#[derive(Debug)]
pub enum NewOrder {
    Buy(BuyOrder),
    Sell(SellOrder),
}

fn create_order_book(mut commands: Commands) {
    commands.insert_resource(OrderBook::default());
}

fn create_quarter_manager(mut commands: Commands) {
    commands.insert_resource(QuarterManger::default());
}

fn create_buy_sell_orchestrator(mut commands: Commands) {
    commands.insert_resource(BuySellOrchestrator::default());
}

#[derive(Component)]
struct StocksToAllocate {
    pub to_allocate: u64,
}

fn spawn_companies(mut commands: Commands, existing_companies: Query<Entity, With<Company>>) {
    struct CompanyTemplate {
        pub ticker: &'static str,
        pub industries: Vec<(f32, Industry)>,
        pub stock_price: Money,
        #[allow(dead_code)]
        pub market_cap: Money,
        pub revenue: Money,
        pub expenses: Money,
        pub dividend_paid: Money,
        pub assets: Money,
        pub outstanding_shares: u64,
    }

    let company_templates = vec![
        CompanyTemplate {
            ticker: "GROUP",
            industries: vec![(1., Industry::Tech)],
            stock_price: 28400,
            market_cap: 11700000000000,
            revenue: 730000000000,
            expenses: 747000000000,
            dividend_paid: 0,
            assets: 190000000000,
            outstanding_shares: 261147000,
        },
        CompanyTemplate {
            ticker: "TSFT",
            industries: vec![
                (0.9, Industry::Tech),
                (0.08, Industry::Entertainment),
                (0.02, Industry::Retail),
            ],
            stock_price: 40400,
            market_cap: 473800000000000,
            revenue: 39838000000000,
            expenses: 22569000000000,
            dividend_paid: 481,
            assets: 47266000000000,
            outstanding_shares: 7435000000,
        },
        CompanyTemplate {
            ticker: "CENT",
            industries: vec![(0.9, Industry::Tech), (0.1, Industry::Entertainment)],
            stock_price: 18100,
            market_cap: 3435000000000,
            revenue: 53288000000000,
            expenses: 34392642014200,
            dividend_paid: 2000,
            assets: 70111550000000,
            outstanding_shares: 12343000000,
        },
        CompanyTemplate {
            ticker: "COBA",
            industries: vec![
                (0.67, Industry::Retail),
                (0.25, Industry::Tech),
                (0.04, Industry::Healthcare),
                (0.03, Industry::Food),
                (0.1, Industry::Entertainment),
            ],
            stock_price: 33600,
            market_cap: 3560000000000,
            revenue: 972570000000,
            expenses: 864220000000,
            dividend_paid: 00,
            assets: 460310000000,
            outstanding_shares: 10501000000,
        },
        CompanyTemplate {
            ticker: "NFLM",
            industries: vec![(1.0, Industry::Entertainment)],
            stock_price: 155000,
            market_cap: 663110000000,
            revenue: 59330000000,
            expenses: 59330000000 - 16230000000,
            dividend_paid: 0,
            assets: 39820000000,
            outstanding_shares: 428239000,
        },
        CompanyTemplate {
            ticker: "USBNK",
            industries: vec![(1.0, Industry::Finance)],
            stock_price: 15500,
            market_cap: 260810000000,
            revenue: 27100000000,
            expenses: 27100000000 - 14340000000,
            dividend_paid: 416,
            assets: 78740000000,
            outstanding_shares: 1690000000,
        },
        CompanyTemplate {
            ticker: "CBANK",
            industries: vec![(1.0, Industry::Finance)],
            stock_price: 119,
            market_cap: 328050000000,
            revenue: 130940000000,
            expenses: 60990000000,
            dividend_paid: 3,
            assets: 660120000000,
            outstanding_shares: 300852631579,
        },
        CompanyTemplate {
            ticker: "TELCO",
            industries: vec![(1.0, Industry::Telecommunications)],
            stock_price: 416,
            market_cap: 47660000000,
            revenue: 22360000000,
            expenses: 22360000000 - 2860000000,
            dividend_paid: 16,
            assets: 18690000000,
            outstanding_shares: 11543000000,
        },
        CompanyTemplate {
            ticker: "KTEL",
            industries: vec![(1.0, Industry::Telecommunications)],
            stock_price: 2763,
            market_cap: 13580000000,
            revenue: 29510000000,
            expenses: 29510000000 - 1730000000,
            dividend_paid: 287,
            assets: 23590000000,
            outstanding_shares: 491651550,
        },
        CompanyTemplate {
            ticker: "RR",
            industries: vec![(1.0, Industry::Mining)],
            stock_price: 9751,
            market_cap: 161480000000,
            revenue: 87010000000,
            expenses: 87010000000 - 25340000000,
            dividend_paid: 700,
            assets: 93330000000,
            outstanding_shares: 1621400000,
        },
        CompanyTemplate {
            ticker: "PPCP",
            industries: vec![(1.0, Industry::Mining)],
            stock_price: 7806,
            market_cap: 197530000000,
            revenue: 86650000000,
            expenses: 86650000000 - 37920000000,
            dividend_paid: 470,
            assets: 79090000000,
            outstanding_shares: 2532000000,
        },
        CompanyTemplate {
            ticker: "EFARM",
            industries: vec![
                (0.7, Industry::Retail),
                (0.25, Industry::Manufacturing),
                (0.05, Industry::Healthcare),
            ],
            stock_price: 7416,
            market_cap: 84170000000,
            revenue: 43410000000,
            expenses: 43410000000 - 3640000000,
            dividend_paid: 198,
            assets: 9250000000,
            outstanding_shares: 1132000000,
        },
        CompanyTemplate {
            ticker: "GAMGO",
            industries: vec![(1.0, Industry::Retail)],
            stock_price: 4032,
            market_cap: 18010000000,
            revenue: 6940000000,
            expenses: 6940000000 + 3460000,
            dividend_paid: 15,
            assets: 7730000000,
            outstanding_shares: 437400000,
        },
        // CompanyTemplate {
        //     ticker: "TEMP",
        //     industries: vec![(1.0, Industry::Entertainment)],
        //     stock_price: 0,
        //     market_cap: 0,
        //     revenue: 0,
        //     expenses: 0,
        //     dividend_paid: 0,
        //     assets: 0,
        //     outstanding_shares: 0,
        // },
    ];

    if existing_companies.iter().next().is_some() {
        return;
    }

    for template in company_templates {
        let entity = commands.spawn_empty().id();

        let shares_to_allocate = if template.outstanding_shares > 10000000 {
            10000000
        } else {
            template.outstanding_shares
        };

        commands.entity(entity).insert((
            CompanyBundle {
                company: Company {
                    ticker: template.ticker.to_string(),
                    existing_shares: template.outstanding_shares,
                    history: vec![CompanyHistory {
                        quarter: 0,
                        assets: template.assets * 100,
                        revenue: template.revenue * 100,
                        expenses: template.expenses * 100,
                        total_shares: template.outstanding_shares,
                        dividend_paid: template.dividend_paid,
                        performance: PerformanceRanking::Average,
                    }],
                    performance_history: vec![],
                    industries: template.industries,
                },
                share_history: ShareHistory::new(template.stock_price),
                wallet: Wallet::default(),
                share_portfolio: SharePortfolio {
                    owned_shares: hashmap! {
                        entity => template.outstanding_shares - shares_to_allocate
                    },
                },
                rng: RngComponent::new(),
                save: Save,
            },
            StocksToAllocate {
                to_allocate: shares_to_allocate,
            },
        ));
    }
}

fn add_rng_to_stock_stuff(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            Or<(With<Company>, With<StockMarketGhost>)>,
            Without<RngComponent>,
        ),
    >,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(RngComponent::new());
    }
}

fn allocate_stocks(
    mut commands: Commands,
    mut companies: Query<(Entity, &mut StocksToAllocate, &mut RngComponent)>,
    ghosts: Query<Entity, (With<StockMarketGhost>, With<SharePortfolio>)>,
    mut portfolios: Query<&mut SharePortfolio, With<StockMarketGhost>>,
) {
    if companies.iter().count() == 0 {
        return;
    }

    let ghosts = ghosts.iter().map(|i| i).collect::<Vec<_>>();

    if ghosts.len() == 0 {
        return;
    }

    for (entity, mut to_allocate, mut rng) in &mut companies {
        // Pick a random ghost to allocate to
        let mut loops = 0;
        while to_allocate.to_allocate > 0 && loops < 100 {
            let ghost = ghosts[rng.usize(0..ghosts.len())];
            const MAX_ALLOCATION: u64 = 100;
            let count = rng.u64(0..=MAX_ALLOCATION.min(to_allocate.to_allocate));

            let mut portfolio = portfolios.get_mut(ghost).unwrap();
            portfolio.add_shares(entity, count);

            to_allocate.to_allocate -= count;

            loops += 1;
        }

        if to_allocate.to_allocate <= 0 {
            commands.entity(entity).remove::<StocksToAllocate>();
        }
    }
}

pub fn step_company_quarter<T: DelegatedRng>(
    quarter: u32,
    company_entity: Entity,
    company: &mut Company,
    share_history: &ShareHistory,
    wallet: &mut Wallet,
    share_portfolio: &mut SharePortfolio,
    order_book: &mut OrderBook,
    rng: &mut T,
) {
    let last = company.history.last().unwrap();

    let next_perf = last.performance.next(rng);

    let mut change_func = |last: Money, range: std::ops::Range<f64>| {
        let change = ((last as f64) * gen_f64_range(rng, &range) as f64) as Money;
        if change > 0 {
            last.checked_add(change).unwrap_or(i64::MAX)
        } else {
            last.checked_sub(change.abs()).unwrap_or(i64::MIN)
        }
    };

    let next_revenue = change_func(last.revenue, next_perf.range());
    let next_expenses = change_func(last.expenses, next_perf.expense_range());
    let next_assets = change_func(last.assets, next_perf.asset_range()) + wallet.balance;

    let profit = next_revenue - next_expenses;

    let next_assets = if profit > 0 {
        next_assets.checked_add(profit).unwrap_or(i64::MAX)
    } else {
        next_assets.checked_sub(profit.abs()).unwrap_or(i64::MIN)
    };

    // Decide if company needs more money
    let capital_target = if profit < 0 {
        profit.abs() as Money
    } else if rng.f32() > 0.9 {
        (profit as f64 * rng.f64()) as Money
    } else {
        0 as Money
    };
    // Raise capital by issuing more shares
    if capital_target > 0 {
        let mut share_price = share_history.price();
        if share_price > 6 {
            share_price -= rng.i64(1..=5);
        }
        let capital_needed = profit.abs();
        let new_shares_qty = capital_needed / share_price;
        company.existing_shares += new_shares_qty as u64;
        order_book.add(NewOrder::Sell(SellOrder::new(
            company_entity,
            new_shares_qty as u64,
            share_price,
            company_entity,
        )));
    }

    if next_assets < 0 {
        // TODO handle bankruptcy
    }

    let next_dividend_paid = 0;

    wallet.balance = 0;

    company.history.push(CompanyHistory {
        quarter,
        assets: next_assets,
        revenue: next_revenue,
        expenses: next_expenses,
        total_shares: last.total_shares,
        dividend_paid: next_dividend_paid,
        performance: next_perf,
    });

    update_company_performance(company, share_history);
}

fn update_company_performance(company: &mut Company, share_history: &ShareHistory) {
    while company.performance_history.len() < company.history.len() {
        let new = CompanyPerformance::new(company, share_history);

        info!(
            "Updated performance for company {} price {} pb_ratio: {}, pe_ratio: {} peg_ratio: {:?}",
            company.ticker, new.stock_price, new.pb_ratio, new.pe_ratio, new.peg_ratio
        );

        company.performance_history.push(new);
    }
}

#[derive(Resource, Deserialize, Serialize, Clone, Reflect)]
#[reflect_value(Deserialize, Serialize, Resource)]
pub struct QuarterManger {
    current_quarter: u32,
    quarter_timer: Timer,
}

impl QuarterManger {
    pub fn percent_complete(&self) -> f32 {
        self.quarter_timer.elapsed().as_secs_f32() / self.quarter_timer.duration().as_secs_f32()
    }
}

impl ToString for QuarterManger {
    fn to_string(&self) -> String {
        // Get year
        let quarter = self.current_quarter as i32 - 1;
        let year = quarter / 4 + 1991;
        let quarter = quarter % 4;
        format!(
            "FY {}-{} Q{} {:0>2}%",
            year,
            year + 1,
            quarter + 1,
            (self.percent_complete() * 100.).floor()
        )
    }
}

impl Default for QuarterManger {
    fn default() -> Self {
        Self {
            current_quarter: 0,
            // quarter_timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
            quarter_timer: Timer::new(Duration::from_mins(5), TimerMode::Repeating),
        }
    }
}

fn tick_quarter(
    time: Res<Time>,
    mut quarter_manager: ResMut<QuarterManger>,
    mut order_book: ResMut<OrderBook>,
    mut companies: Query<(
        Entity,
        &mut Company,
        &ShareHistory,
        &mut SharePortfolio,
        &mut Wallet,
        &mut RngComponent,
    )>,
) {
    if quarter_manager
        .quarter_timer
        .tick(time.delta())
        .just_finished()
        || quarter_manager.current_quarter == 0
    {
        quarter_manager.current_quarter += 1;

        for (entity, mut company, share_history, mut portfolio, mut wallet, rng) in
            companies.iter_mut()
        {
            let rng = rng.into_inner();

            step_company_quarter(
                quarter_manager.current_quarter,
                entity,
                &mut company,
                &share_history,
                &mut wallet,
                &mut portfolio,
                &mut order_book,
                rng,
            );
        }
    }
}

fn update_company_price_cache(mut companies: Query<&mut ShareHistory, Changed<ShareHistory>>) {
    for mut share_history in &mut companies {
        share_history.update_cached_price();
    }
}

#[derive(Default, Component, Deserialize, Serialize, Clone, Reflect)]
#[reflect_value(Component, Serialize, Deserialize)]
pub struct StockMarketGhost;

#[derive(Default, Bundle)]
pub struct StockMarketGhostBundle {
    pub ghost: StockMarketGhost,
    pub wallet: Wallet,
    pub share_portfolio: SharePortfolio,
    pub rng: RngComponent,
    pub save: Save,
    pub ai: StockMarketAI,
}

fn spawn_ghosts(
    mut commands: Commands,
    mut global_rng: ResMut<GlobalRng>,
    existing_ghosts: Query<Entity, With<StockMarketGhost>>,
) {
    if existing_ghosts.iter().next().is_some() {
        return;
    }

    for _ in 0..100 {
        let mut rng = RngComponent::with_seed(global_rng.u64(u64::MIN..u64::MAX));
        commands.spawn(StockMarketGhostBundle {
            wallet: Wallet {
                balance: rng.i64(10000000..100000000),
                ..default()
            },
            ai: StockMarketAI::new_from_rng(&mut rng),
            rng,
            ..default()
        });
    }
}

#[derive(Resource, Deserialize, Serialize, Clone, Reflect)]
#[reflect_value(Deserialize, Serialize, Resource)]
pub struct BuySellOrchestrator {
    buy_timer: Timer,
}

impl Default for BuySellOrchestrator {
    fn default() -> Self {
        Self {
            buy_timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
        }
    }
}

#[derive(Component, Deserialize, Serialize, Clone, Reflect)]
#[reflect_value(Component, Deserialize, Serialize)]
pub struct StockMarketAI {
    pe_weight: f32,
    pb_weight: f32,
    peg_weight: f32,
}

impl StockMarketAI {
    pub fn new(pe_weight: f32, pb_weight: f32, peg_weight: f32) -> Self {
        Self {
            pe_weight,
            pb_weight,
            peg_weight,
        }
    }

    pub fn new_from_rng<T: DelegatedRng>(rng: &mut T) -> Self {
        lazy_static! {
            static ref WEIGHT_TABLES: WalkerTable = WalkerTable::new(&[5, 10, 85]);
        }

        const POSSIBLE_WEIGHTS: [[f32; 3]; 3] =
            [[0.05, 0.05, 0.9], [0.1, 0.2, 0.3], [0.33, 0.33, 0.33]];

        let weights = POSSIBLE_WEIGHTS[WEIGHT_TABLES.next_rng(rng)];

        match rng.i8(0..=2) {
            0 => Self::new(weights[0], weights[1], weights[2]),
            1 => Self::new(weights[0], weights[2], weights[1]),
            2 => Self::new(weights[1], weights[0], weights[2]),
            _ => unreachable!(),
        }
    }

    fn get_buy_threshold(&self, rank: &CompanyRank) -> BuyThreshold {
        const MAX_SCORE_PART: f32 = 100.;
        let pe_score = (1. - rank.pe_percentile) * MAX_SCORE_PART * self.pe_weight;
        let pb_score = (1. - rank.pb_percentile) * MAX_SCORE_PART * self.pb_weight;
        let peg_score = (1. - rank.peg_percentile) * MAX_SCORE_PART * self.peg_weight;

        BuyThreshold::from_score(pe_score + pb_score + peg_score)
    }
}

impl Default for StockMarketAI {
    fn default() -> Self {
        Self::new(0.3, 0.3, 0.4)
    }
}

fn generate_buy_sell_activity(
    time: Res<Time>,
    mut orchestrator: ResMut<BuySellOrchestrator>,
    mut order_book: ResMut<OrderBook>,
    mut buyer_sellers: Query<(
        Entity,
        &mut SharePortfolio,
        &mut Wallet,
        &mut RngComponent,
        &StockMarketAI,
    )>,
    companies: Query<(Entity, &Company, &ShareHistory)>,
) {
    if !orchestrator.buy_timer.tick(time.delta()).finished() {
        return;
    }

    struct CompanySet<'a> {
        entity: Entity,
        company: &'a Company,
        performance: CompanyPerformance,
    }

    let companies = companies
        .iter()
        .map(|i| {
            let (entity, company, share_history) = i;
            let performance = CompanyPerformance::new(company, share_history);
            CompanySet {
                entity,
                company,
                performance,
            }
        })
        .collect::<Vec<_>>();

    let mut company_lookup = Vec::with_capacity(companies.len());
    for company in 0..companies.len() {
        company_lookup.push(company);
    }

    let mut pe_ranking = company_lookup.clone().into_iter().collect::<Vec<_>>();
    pe_ranking.sort_by(|a, b| {
        companies[*a]
            .performance
            .pe_ratio
            .partial_cmp(&companies[*b].performance.pe_ratio)
            .unwrap()
    });

    let mut pb_ranking = company_lookup.clone().into_iter().collect::<Vec<_>>();
    pb_ranking.sort_by(|a, b| {
        companies[*a]
            .performance
            .pb_ratio
            .partial_cmp(&companies[*b].performance.pb_ratio)
            .unwrap()
            .reverse()
    });

    let mut peg_ranking = company_lookup.clone().into_iter().collect::<Vec<_>>();
    peg_ranking.sort_by(|a, b| {
        companies[*a]
            .performance
            .peg_ratio
            .partial_cmp(&companies[*b].performance.peg_ratio)
            .unwrap()
    });

    let mut company_rankings = HashMap::new();
    for (i, company) in company_lookup.iter().enumerate() {
        let pe_rank = pe_ranking.iter().position(|x| *x == i).unwrap();
        let pb_rank = pb_ranking.iter().position(|x| *x == i).unwrap();
        let peg_rank = peg_ranking.iter().position(|x| *x == i).unwrap();
        company_rankings.insert(
            *company,
            CompanyRank {
                pe_percentile: (pe_rank + 1) as f32 / companies.len() as f32,
                pb_percentile: (pb_rank + 1) as f32 / companies.len() as f32,
                peg_percentile: (peg_rank + 1) as f32 / companies.len() as f32,
            },
        );
    }

    const MAX_ORDER_SIZE: u64 = 100;

    for (entity, mut portfolio, mut wallet, rng, ai) in buyer_sellers.iter_mut() {
        let rng = rng.into_inner();

        for (i, company) in companies.iter().enumerate() {
            let company_rank = company_rankings.get(&i).unwrap();

            let buy_threshold = ai.get_buy_threshold(company_rank);

            if rng.f32() > buy_threshold.buy_prob() {
                continue;
            }

            let price =
                (company.performance.stock_price as f32 * buy_threshold.price_modifier()) as Money;
            let price = (price
                + (price as f32 * gen_f32_range(rng, &(-0.05..0.05))).floor() as Money)
                .max(1);

            // Don't spend more than 1% of wallet
            let money_available = (wallet.balance as f32 * 0.01).floor() as Money;

            if money_available < price {
                continue;
            }

            let max_quantity = MAX_ORDER_SIZE.min(money_available as u64 / price as u64);

            let quantity = rng.u64(0..=max_quantity);

            if quantity == 0 {
                continue;
            }

            wallet.balance -= quantity as i64 * price;
            let order = BuyOrder::new(company.entity, quantity, price, entity);

            order_book.add(NewOrder::Buy(order));
        }

        for (i, company) in companies.iter().enumerate() {
            let shares = match portfolio.owned_shares.get(&company.entity) {
                Some(shares) => *shares,
                None => continue,
            };

            if shares == 0 {
                continue;
            }

            let rank = company_rankings.get(&i).unwrap();

            let sell_threshold = ai.get_buy_threshold(rank).invert();

            if rng.f32() < sell_threshold.buy_prob() {
                continue;
            }

            let price =
                (company.performance.stock_price as f32 * sell_threshold.price_modifier()) as Money;

            let price = (price
                + (price as f32 * gen_f32_range(rng, &(0.01..0.05))).floor() as Money)
                .max(1);

            let max_sell = MAX_ORDER_SIZE.min(shares);

            let quantity = rng.u64(0..=max_sell);

            if quantity == 0 {
                continue;
            }

            portfolio.remove_shares(company.entity, quantity);
            order_book.add(NewOrder::Sell(SellOrder::new(
                company.entity,
                quantity,
                price,
                entity,
            )));
        }
    }
}

fn process_orders(
    time: Res<Time>,
    quarter_manager: Res<QuarterManger>,
    order_book: ResMut<OrderBook>,
    mut share_history: Query<&mut ShareHistory>,
    mut wallets: Query<&mut Wallet>,
    mut share_portfolios: Query<&mut SharePortfolio>,
) {
    let order_book = order_book.into_inner();

    {
        let buy_orders = &mut order_book.buy_orders;
        let sell_orders = &mut order_book.sell_orders;
        for (i, buy_order) in buy_orders.iter_mut().enumerate() {
            buy_order.lifetime += time.delta();

            let sell_orders = match sell_orders.get_mut(&buy_order.company) {
                Some(orders) => orders,
                None => continue,
            };

            for sell_order in sell_orders {
                // Since all sell orders are sorted by price, we can break if the the sell order is more costly
                if buy_order.price < sell_order.price {
                    break;
                }

                if sell_order.remaining_quantity == 0 {
                    continue;
                }

                let quantity = buy_order
                    .remaining_quantity
                    .min(sell_order.remaining_quantity);

                let mut seller_wallet = match wallets.get_mut(sell_order.seller) {
                    Ok(wallet) => wallet,
                    Err(_) => continue,
                };

                let mut buyer_portfolio = match share_portfolios.get_mut(buy_order.buyer) {
                    Ok(portfolio) => portfolio,
                    Err(_) => break,
                };

                buyer_portfolio.add_shares(sell_order.company, quantity);
                seller_wallet.balance += quantity as i64 * sell_order.price;

                buy_order.remaining_quantity -= quantity;
                sell_order.remaining_quantity -= quantity;

                if let Ok(mut share_history) = share_history.get_mut(sell_order.company) {
                    share_history.add_entry(sell_order.price, quantity);
                }

                if buy_order.remaining_quantity == 0 {
                    break;
                }
            }
        }
    }

    // Update sell order lifetimes
    for orders in order_book.sell_orders.values_mut() {
        for order in orders {
            order.lifetime += time.delta();
        }
    }

    const PULL_TIME: Duration = Duration::from_secs(60);

    order_book.buy_orders.retain(|order| {
        if order.remaining_quantity == 0 {
            return false;
        }

        if order.lifetime > PULL_TIME || share_portfolios.get(order.buyer).is_err() {
            // Attempt to refund the buyer
            if let Ok(mut wallet) = wallets.get_mut(order.buyer) {
                wallet.balance += order.remaining_quantity as i64 * order.price;
            }
            return false;
        }

        true
    });

    order_book.sell_orders.retain(|_, orders| {
        orders.retain(|order| {
            if order.remaining_quantity == 0 {
                return false;
            }

            if order.lifetime > PULL_TIME || wallets.get(order.seller).is_err() {
                // return shares to seller
                if let Ok(mut portfolio) = share_portfolios.get_mut(order.seller) {
                    portfolio.add_shares(order.company, order.remaining_quantity);
                }
                return false;
            }

            true
        });

        !orders.is_empty()
    });
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, u64};

    use shared_deps::bevy_turborand::{prelude::RngPlugin, GlobalRng};

    use super::*;

    #[test]
    fn test_tick_quarter() {
        const SIM_COUNT: i128 = 500;
        const QUARTER_COUNT: u32 = 20;

        let mut quarter_sum = HashMap::new();

        for i in 0..=SIM_COUNT {
            let mut time = Time::<()>::default();
            time.advance_by(Duration::from_days(1));

            let mut app = App::new();
            app.insert_resource(GlobalRng::with_seed(i as u64));
            app.insert_resource(time);

            fn spawn_test_companies(mut commands: Commands, mut global_rng: ResMut<GlobalRng>) {
                let company = Company {
                    ticker: "TEST".to_string(),
                    existing_shares: 1000,
                    history: vec![CompanyHistory {
                        quarter: 0,
                        assets: 100000,
                        revenue: 10000,
                        expenses: 9000,
                        total_shares: 1000,
                        dividend_paid: 0,
                        performance: PerformanceRanking::Average,
                    }],
                    performance_history: vec![],
                    industries: vec![(1., Industry::Tech)],
                };

                commands.spawn(CompanyBundle {
                    company,
                    share_history: ShareHistory::new(100),
                    wallet: Wallet::default(),
                    share_portfolio: SharePortfolio::default(),
                    rng: RngComponent::with_seed(global_rng.u64(u64::MIN..=u64::MAX)),
                    save: Save,
                });
            }

            app.add_systems(
                Startup,
                (
                    create_order_book,
                    create_quarter_manager,
                    create_buy_sell_orchestrator,
                    spawn_test_companies,
                )
                    .chain(),
            );
            app.add_systems(
                Update,
                (add_rng_to_stock_stuff, allocate_stocks, tick_quarter).chain(),
            );

            loop {
                app.update();

                let quarter_manager = app.world().get_resource::<QuarterManger>().unwrap();
                let current_quarter = quarter_manager.current_quarter;

                for company in app.world_mut().query::<&Company>().iter(app.world()) {
                    let quarter = current_quarter;
                    let company_map = quarter_sum.entry(quarter).or_insert(HashMap::new());
                    *company_map.entry(company.ticker.clone()).or_insert(0i128) +=
                        company.history.last().unwrap().assets as i128;
                }

                if current_quarter >= QUARTER_COUNT {
                    break;
                }

                let mut time = app.world_mut().get_resource_mut::<Time>().unwrap();
                time.advance_by(Duration::from_days(1));
            }
        }

        let tickers = quarter_sum
            .iter()
            .flat_map(|(_, map)| map.keys())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        struct QuarterCheck {
            quarter: u32,
            min_change: f64,
            max_change: f64,
        }

        let checks = vec![
            QuarterCheck {
                quarter: 2,
                min_change: 0.01,
                max_change: 0.02,
            },
            QuarterCheck {
                quarter: 5,
                min_change: 0.05,
                max_change: 0.08,
            },
            QuarterCheck {
                quarter: 20,
                min_change: 0.25,
                max_change: 0.35,
            },
        ];

        for check in checks {
            for ticker in &tickers {
                let first_quarter_avg =
                    quarter_sum.get(&1).unwrap().get(*ticker).unwrap() / SIM_COUNT;
                let current_quarter_avg = quarter_sum
                    .get(&check.quarter)
                    .unwrap()
                    .get(*ticker)
                    .unwrap()
                    / SIM_COUNT;

                let change = (current_quarter_avg as f64 - first_quarter_avg as f64)
                    / first_quarter_avg as f64;
                assert!(
                    change >= check.min_change,
                    "Company: {}, Q{} change {} >= {} ",
                    ticker,
                    check.quarter,
                    change,
                    check.min_change
                );
                assert!(
                    change <= check.max_change,
                    "Company: {}, Q{} change {} <= {} ",
                    ticker,
                    check.quarter,
                    change,
                    check.max_change
                );
            }
        }
    }

    #[test]
    fn test_generate_buy_activity() {
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_days(1));

        let mut app = App::new();
        app.add_plugins(RngPlugin::default());
        app.insert_resource(time);
        app.add_systems(
            Startup,
            (
                create_order_book,
                create_quarter_manager,
                create_buy_sell_orchestrator,
                spawn_companies,
                spawn_ghosts,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                add_rng_to_stock_stuff,
                allocate_stocks,
                tick_quarter,
                generate_buy_sell_activity,
            )
                .chain(),
        );
        app.update();

        // Check book validate for tests
        let order_book = app.world().get_resource::<OrderBook>().unwrap();

        assert!(!order_book.buy_orders.is_empty());
    }

    #[test]
    fn test_process_orders() {
        let time = Time::<()>::default();

        let mut app = App::new();
        app.add_plugins(RngPlugin::default());
        app.insert_resource(time);
        app.add_systems(
            Startup,
            (
                create_order_book,
                create_quarter_manager,
                create_buy_sell_orchestrator,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (add_rng_to_stock_stuff, allocate_stocks, process_orders).chain(),
        );

        fn gen_test_orders(mut commands: Commands, mut order_book: ResMut<OrderBook>) {
            // Spawn test company

            let company_id = commands
                .spawn(CompanyBundle {
                    company: Company {
                        ticker: "TEST".to_string(),
                        existing_shares: 1000,
                        history: vec![CompanyHistory {
                            quarter: 0,
                            assets: 100000,
                            revenue: 10000,
                            expenses: 5000,
                            total_shares: 1000,
                            dividend_paid: 0,
                            performance: PerformanceRanking::Average,
                        }],
                        performance_history: vec![],
                        industries: vec![(1., Industry::Tech)],
                    },
                    share_history: ShareHistory::new(100),
                    wallet: Wallet::default(),
                    share_portfolio: SharePortfolio::default(),
                    rng: RngComponent::new(),
                    save: Save,
                })
                .id();

            let selling_ghost_id = commands
                .spawn(StockMarketGhostBundle {
                    wallet: Wallet {
                        balance: 1000000,
                        ..default()
                    },
                    rng: RngComponent::new(),
                    ..default()
                })
                .id();

            let buying_ghost_id = commands
                .spawn(StockMarketGhostBundle {
                    wallet: Wallet {
                        balance: 1000000,
                        ..default()
                    },
                    rng: RngComponent::new(),
                    share_portfolio: SharePortfolio::default(),
                    ..default()
                })
                .id();

            order_book.add(NewOrder::Sell(SellOrder::new(
                company_id,
                500,
                100,
                selling_ghost_id,
            )));

            order_book.add(NewOrder::Sell(SellOrder::new(
                company_id,
                500,
                200,
                selling_ghost_id,
            )));

            order_book.add(NewOrder::Buy(BuyOrder::new(
                company_id,
                1000,
                100,
                buying_ghost_id,
            )));
        }

        app.add_systems(PreUpdate, gen_test_orders.run_if(run_once()));
        app.update();

        // Check book validate for tests
        let order_book = app.world().get_resource::<OrderBook>().unwrap();

        assert_eq!(order_book.buy_orders[0].remaining_quantity, 500,);
        order_book.sell_orders.values().for_each(|orders| {
            assert_eq!(orders.len(), 1);
            assert_eq!(orders[0].price, 200);
        });
    }

    #[test]
    fn test_rate_company() {
        let company = Company {
            ticker: "TELCO".to_string(),
            existing_shares: 11543000000,
            history: vec![CompanyHistory {
                quarter: 0,
                assets: 1869000000000,
                revenue: 2236000000000,
                expenses: 2236000000000 - 286000000000,
                total_shares: 11543000000,
                dividend_paid: 18,
                performance: PerformanceRanking::Average,
            }],
            performance_history: vec![],
            industries: vec![(1., Industry::Tech)],
        };
        let share_history = ShareHistory::new(414);

        let performance = CompanyPerformance::new(&company, &share_history);

        assert_eq!(performance.stock_price, 414);
        assert_eq!(format!("{:.2}", performance.pb_ratio), "2.56");
        assert_eq!(format!("{:.2}", performance.pe_ratio), "16.71");

        let ai = StockMarketAI::default();

        let rank = CompanyRank {
            pe_percentile: 0.01,
            pb_percentile: 0.01,
            peg_percentile: 0.01,
        };

        let buy_threshold = ai.get_buy_threshold(&rank);

        assert_eq!(buy_threshold, BuyThreshold::Must);
    }

    #[test]
    fn test_order_book_sell_ordered() {
        let mut app = App::new();

        let company_entity = app.world_mut().spawn_empty().id();
        let seller_entity = app.world_mut().spawn_empty().id();

        let mut order_book = OrderBook::default();

        let mut prices = vec![5, 2, 7, 2, 9, 10, 3];
        for price in &prices {
            order_book.add(NewOrder::Sell(SellOrder::new(
                company_entity,
                100,
                *price,
                seller_entity,
            )));
        }

        let sell_orders = order_book.get_sell_orders(company_entity);

        prices.sort();
        for (i, order) in sell_orders.iter().enumerate() {
            assert_eq!(order.price, prices[i]);
        }
    }
}

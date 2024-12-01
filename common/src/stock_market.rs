use bevy::{prelude::*, utils::HashMap};
use bevy_turborand::{DelegatedRng, GlobalRng};
use moonshine_save::save::Save;
use rand::Rng;
use serde::{Deserialize, Serialize};
use weighted_rand::{
    builder::{NewBuilder, WalkerTableBuilder},
    table::WalkerTable,
};

use crate::{
    money::{Money, Wallet},
    name::EntityName,
    sardip_save::SardipLoadingState,
    simulation::SimulationUpdate,
};

pub struct StockMarketPlugin;

impl Plugin for StockMarketPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CompanyShare>()
            .register_type::<HashMap<ShareKey, CompanyShare>>()
            .register_type_data::<HashMap<ShareKey, CompanyShare>, ReflectSerialize>()
            .register_type_data::<HashMap<ShareKey, CompanyShare>, ReflectDeserialize>()
            .register_type::<SharePortfolio>()
            .register_type::<Company>()
            .register_type::<Order>()
            .register_type::<BuySellOrder>()
            .add_systems(Update, (add_company_shares_cache, update_share_cache))
            .add_systems(
                OnEnter(SardipLoadingState::Loaded),
                (
                    initialize_companies,
                    initialize_order_book.run_if(not(resource_exists::<OrderBook>)),
                ),
            )
            .add_systems(SimulationUpdate, generate_orders);
    }
}

type ShareKey = u32;

#[derive(Default, Reflect, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[reflect_value(Deserialize, Serialize, PartialEq)]
pub struct CompanyShare {
    pub quantity: u64,
    pub owner: ShareKey,
}

#[derive(Default, Reflect, Component)]
#[reflect(Component)]
pub struct Company {
    pub company_ticker: String,
    pub company_shares: HashMap<ShareKey, CompanyShare>,
    pub available_shares: u64,
}

#[derive(Bundle)]
pub struct CompanyBundle {
    pub name: EntityName,
    pub company: Company,
    pub wallet: Wallet,
    pub save: Save,
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct SharePortfolio {
    pub share_key: ShareKey,
}

impl SharePortfolio {
    pub fn new_player_portfolio() -> Self {
        Self {
            share_key: rand::thread_rng().gen_range(PLAYER_SHARE_KEY_RANGE),
        }
    }
}

const PLAYER_SHARE_KEY_RANGE: std::ops::Range<u32> = 1..1000;
// const COMPUTER_SHARE_KEY_RANGE: std::ops::Range<u32> = 1000..100000;

#[derive(Default, Component)]
pub struct SharePortfolioCache {
    pub company_shares: HashMap<String, CompanyShare>,
}

fn add_company_shares_cache(
    mut commands: Commands,
    company_query: Query<Entity, With<SharePortfolio>>,
) {
    for company_entity in company_query.iter() {
        commands
            .entity(company_entity)
            .insert(SharePortfolioCache::default());
    }
}

fn update_share_cache(
    changed_companies: Query<&Company, Changed<Company>>,
    mut portfolio: Query<(&SharePortfolio, &mut SharePortfolioCache)>,
) {
    let mut updates = HashMap::new();
    for (portfolio, cache) in portfolio.iter_mut() {
        updates.insert(portfolio.share_key, cache);
    }

    for company in changed_companies.iter() {
        for (key, share) in company.company_shares.iter() {
            if let Some(cache) = updates.get_mut(key) {
                cache
                    .company_shares
                    .insert(company.company_ticker.clone(), share.clone());
            }
        }
    }
}

struct CompanyTemplate {
    pub name: &'static str,
    pub ticker: &'static str,
    pub shares: u64,
    pub money: Money,
}

fn initialize_companies(mut commands: Commands, existing_companies: Query<&Company>) {
    const COMPANY_TEMPLATES: [CompanyTemplate; 3] = [
        CompanyTemplate {
            name: "Atlassian",
            ticker: "TEAM",
            shares: 259717000,
            money: 2330000000,
        },
        CompanyTemplate {
            name: "Microsoft",
            ticker: "MSFT",
            shares: 7433000000,
            money: 112150000000,
        },
        CompanyTemplate {
            name: "Google",
            ticker: "GOOGL",
            shares: 12343000000,
            money: 149560000000,
        },
    ];

    for template in COMPANY_TEMPLATES.iter() {
        if existing_companies
            .iter()
            .any(|company| company.company_ticker == template.ticker)
        {
            continue;
        }

        let company_shares = HashMap::new();

        commands.spawn(CompanyBundle {
            name: EntityName::new(template.name),
            company: Company {
                company_ticker: template.ticker.to_string(),
                company_shares,
                available_shares: template.shares,
            },
            wallet: Wallet {
                balance: template.money,
            },
            save: Save,
        });
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect)]
#[reflect_value(Deserialize, Serialize, PartialEq)]
pub struct Order {
    pub ticker: String,
    pub quantity: u64,
    pub price: Money,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Reflect)]
#[reflect_value(Deserialize, Serialize, PartialEq)]
pub enum BuySellOrder {
    Buy(Order),
    Sell(Order),
}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct OrderBook {
    pub orders: Vec<BuySellOrder>,
}

fn initialize_order_book(mut commands: Commands) {
    commands.insert_resource(OrderBook::default());
}

struct OrderBookGeneratorTimer {
    timer: Timer,
}

impl Default for OrderBookGeneratorTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5., TimerMode::Repeating),
        }
    }
}

fn generate_orders(
    mut local: Local<OrderBookGeneratorTimer>,
    time: Res<Time>,
    mut rng: ResMut<GlobalRng>,
    // order_book: ResMut<OrderBook>,
) {
    if !local.timer.tick(time.delta()).just_finished() {
        return;
    }

    const COUNTS: [(u32, u32); 2] = [(1, 10), (10, 20)];

    lazy_static! {
        static ref WA_TABLE: WalkerTable = {
            const INDEX_WEIGHTS: [u32; 2] = [1, 1];
            WalkerTableBuilder::new(&INDEX_WEIGHTS).build()
        };
    };

    let (min, max) = COUNTS[WA_TABLE.next()];
    let count = rng.u32(min..=max);

    for _ in 0..count {}
}

// Same factory as CLI but for Kafka context
use crypto_balance_core::{
    // Import adapters
    adapters::{
        config::app_config::CONFIG, exchange::binance_factory::BinanceAccountFactory,
        exchange::kraken_factory::KrakenFactory,
        exchange::spreadsheet_balance_repository::SpreadsheetBalanceRepository,
        sheets::spreadsheet_manager::SpreadsheetManager,
    },
    application::service::CryptoBalanceApplicationService,
    // Import existing routines and implementations
    application::{
        debank::debank_routine::DebankRoutine, exchange::binance_use_cases::BinanceUseCases,
        exchange::exchange_balances_routine::ExchangeBalancesRoutine,
        exchange::kraken_use_cases::KrakenUseCases, price::token_prices::TokenPricesRoutine,
    },

    ports::{
        application_service::ApplicationService, balance_repository::BalanceRepository,
        routine::Routine,
    },
};

use std::sync::Arc;

pub struct ApplicationServiceFactory;

impl ApplicationServiceFactory {
    pub async fn create() -> Result<Arc<dyn ApplicationService>, Box<dyn std::error::Error>> {
        let routines = Self::create_routines().await;
        let app_service = CryptoBalanceApplicationService::new(routines);
        Ok(Arc::new(app_service))
    }

    async fn create_routines() -> Vec<Box<dyn Routine>> {
        let spreadsheet_manager = Arc::new(SpreadsheetManager::new(CONFIG.sheets.clone()).await);

        let balance_repository: Arc<dyn BalanceRepository> = Arc::new(
            SpreadsheetBalanceRepository::new(Arc::clone(&spreadsheet_manager)),
        );

        vec![
            Box::new(DebankRoutine::new(
                CONFIG.blockchain.airdrops.evm.clone(),
                Arc::clone(&spreadsheet_manager),
            )),
            Box::new(TokenPricesRoutine::new(Arc::clone(&spreadsheet_manager))),
            Box::new(ExchangeBalancesRoutine::new(
                BinanceUseCases::new(BinanceAccountFactory::new(CONFIG.binance.clone())),
                Arc::clone(&balance_repository),
            )),
            Box::new(ExchangeBalancesRoutine::new(
                KrakenUseCases::new(KrakenFactory::new(CONFIG.kraken.clone())),
                Arc::clone(&balance_repository),
            )),
        ]
    }
}

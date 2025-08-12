use crypto_balance_core::{
    application::service::CryptoBalanceApplicationService,
    ports::{application_service::ApplicationService, routine::Routine, balance_repository::BalanceRepository},
    
    // Import existing routines and implementations
    application::{
        debank::debank_routine::DebankRoutine,
        exchange::exchange_balances_routine::ExchangeBalancesRoutine,
        exchange::binance_use_cases::BinanceUseCases,
        exchange::kraken_use_cases::KrakenUseCases,
        price::token_prices::TokenPricesRoutine,
    },
    
    // Import adapters
    adapters::{
        exchange::binance_factory::BinanceAccountFactory,
        exchange::kraken_factory::KrakenFactory,
        exchange::spreadsheet_balance_repository::SpreadsheetBalanceRepository,
        sheets::spreadsheet_manager::SpreadsheetManager,
        config::app_config::CONFIG,
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
        let spreadsheet_manager = Arc::new(
            SpreadsheetManager::new(CONFIG.sheets.clone()).await
        );

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
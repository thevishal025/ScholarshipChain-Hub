#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, log, symbol_short, Address, Env, String, Symbol,
};

// Structure to store scholarship application details
#[contracttype]
#[derive(Clone)]
pub struct Scholarship {
    pub scholar_id: u64,
    pub student: Address,
    pub gpa_score: u64, // GPA multiplied by 100 (e.g., 3.85 = 385)
    pub apply_time: u64,
    pub is_approved: bool,
    pub amount_awarded: u64, // Scholarship amount in stroops
}

// Structure to track scholarship statistics
#[contracttype]
#[derive(Clone)]
pub struct ScholarshipStats {
    pub total_applications: u64,
    pub approved_count: u64,
    pub pending_count: u64,
    pub total_disbursed: u64, // Total amount disbursed
}

// Mapping scholar_id to Scholarship
#[contracttype]
pub enum ScholarBook {
    Scholar(u64),
}

// Counter for generating unique scholar IDs
const SCHOLAR_COUNT: Symbol = symbol_short!("S_COUNT");

// Symbol for storing scholarship statistics
const SCHOLAR_STATS: Symbol = symbol_short!("S_STATS");

// Minimum GPA requirement (3.0 = 300)
const MIN_GPA: u64 = 300;

#[contract]
pub struct ScholarshipContract;

#[contractimpl]
impl ScholarshipContract {
    // Function to apply for scholarship with academic performance proof
    pub fn apply_scholarship(env: Env, student: Address, gpa_score: u64) -> u64 {
        // Require student authentication
        student.require_auth();

        // Validate GPA score (must be between 0-400, representing 0.0-4.0)
        if gpa_score > 400 {
            log!(&env, "Invalid GPA score. Must be between 0-400");
            panic!("Invalid GPA score!");
        }

        // Get and increment scholar counter
        let mut scholar_count: u64 = env.storage().instance().get(&SCHOLAR_COUNT).unwrap_or(0);
        scholar_count += 1;

        // Get current timestamp
        let timestamp = env.ledger().timestamp();

        // Create new scholarship application
        let new_application = Scholarship {
            scholar_id: scholar_count,
            student: student.clone(),
            gpa_score,
            apply_time: timestamp,
            is_approved: false,
            amount_awarded: 0,
        };

        // Store application in blockchain
        env.storage()
            .instance()
            .set(&ScholarBook::Scholar(scholar_count), &new_application);

        // Update scholar counter
        env.storage().instance().set(&SCHOLAR_COUNT, &scholar_count);

        // Update statistics
        let mut stats = Self::get_scholarship_stats(env.clone());
        stats.total_applications += 1;
        stats.pending_count += 1;
        env.storage().instance().set(&SCHOLAR_STATS, &stats);

        // Extend storage TTL
        env.storage().instance().extend_ttl(5000, 5000);

        log!(
            &env,
            "Scholarship application submitted with ID: {}",
            scholar_count
        );

        scholar_count
    }

    // Function to approve scholarship and disburse amount based on GPA
    pub fn approve_scholarship(env: Env, scholar_id: u64) {
        // Get scholarship application
        let mut application = Self::get_scholarship_details(env.clone(), scholar_id);

        if application.scholar_id == 0 {
            log!(&env, "Scholarship application not found!");
            panic!("Application not found!");
        }

        // Check if already approved
        if application.is_approved {
            log!(&env, "Scholarship already approved!");
            panic!("Already approved!");
        }

        // Check GPA eligibility
        if application.gpa_score < MIN_GPA {
            log!(&env, "GPA below minimum requirement (3.0)");
            panic!("GPA below minimum!");
        }

        // Calculate scholarship amount based on GPA
        // GPA 3.0-3.49 = 1000 XLM, 3.5-3.79 = 1500 XLM, 3.8+ = 2000 XLM
        let amount = if application.gpa_score >= 380 {
            2000_0000000 // 2000 XLM in stroops
        } else if application.gpa_score >= 350 {
            1500_0000000 // 1500 XLM in stroops
        } else {
            1000_0000000 // 1000 XLM in stroops
        };

        // Update application
        application.is_approved = true;
        application.amount_awarded = amount;

        // Store updated application
        env.storage()
            .instance()
            .set(&ScholarBook::Scholar(scholar_id), &application);

        // Update statistics
        let mut stats = Self::get_scholarship_stats(env.clone());
        stats.approved_count += 1;
        stats.pending_count -= 1;
        stats.total_disbursed += amount;
        env.storage().instance().set(&SCHOLAR_STATS, &stats);

        // Extend storage TTL
        env.storage().instance().extend_ttl(5000, 5000);

        log!(
            &env,
            "Scholarship ID: {} approved with amount: {}",
            scholar_id,
            amount
        );
    }

    // Function to get scholarship application details
    pub fn get_scholarship_details(env: Env, scholar_id: u64) -> Scholarship {
        let key = ScholarBook::Scholar(scholar_id);

        env.storage().instance().get(&key).unwrap_or(Scholarship {
            scholar_id: 0,
            student: Address::from_string(&String::from_str(
                &env,
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            )),
            gpa_score: 0,
            apply_time: 0,
            is_approved: false,
            amount_awarded: 0,
        })
    }

    // Function to get scholarship statistics
    pub fn get_scholarship_stats(env: Env) -> ScholarshipStats {
        env.storage()
            .instance()
            .get(&SCHOLAR_STATS)
            .unwrap_or(ScholarshipStats {
                total_applications: 0,
                approved_count: 0,
                pending_count: 0,
                total_disbursed: 0,
            })
    }
}

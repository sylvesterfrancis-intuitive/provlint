pub mod schema;
pub mod security;
pub mod best_practice;

use crate::rule::RuleRegistry;

pub fn register_all(registry: &mut RuleRegistry) {
    // Schema rules
    registry.register(Box::new(schema::SchDuplicateDirective));
    registry.register(Box::new(schema::SchAutoYaSTStructure));
    registry.register(Box::new(schema::SchAutoinstallStructure));

    // Security rules
    registry.register(Box::new(security::SecPlaintextPassword));
    registry.register(Box::new(security::SecSelinuxDisabled));
    registry.register(Box::new(security::SecFirewallDisabled));
    registry.register(Box::new(security::SecPermitRootLogin));
    registry.register(Box::new(security::SecWeakHash));
    registry.register(Box::new(security::SecHttpRepo));
    registry.register(Box::new(security::SecEmptyPassword));

    // Best practice rules
    registry.register(Box::new(best_practice::BpNoSwap));
    registry.register(Box::new(best_practice::BpNoBootloaderPassword));
    registry.register(Box::new(best_practice::BpMissingHostname));
    registry.register(Box::new(best_practice::BpNoNtp));
    registry.register(Box::new(best_practice::BpDeprecatedDirective));
}

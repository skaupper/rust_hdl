use crate::ast::{AnyDesignUnit, AnyPrimaryUnit, AnySecondaryUnit};

pub fn get_primary_unit_name(pu: &AnyPrimaryUnit) -> String {
    match pu {
        AnyPrimaryUnit::Configuration(cfg) => cfg.ident.tree.item.name_utf8(),
        AnyPrimaryUnit::Context(ctx) => ctx.ident.tree.item.name_utf8(),
        AnyPrimaryUnit::Entity(ent) => ent.ident.tree.item.name_utf8(),
        AnyPrimaryUnit::Package(pkg) => pkg.ident.tree.item.name_utf8(),
        AnyPrimaryUnit::PackageInstance(pkg_inst) => pkg_inst.ident.tree.item.name_utf8(),
    }
}

pub fn get_secondary_unit_name(su: &AnySecondaryUnit) -> String {
    match su {
        AnySecondaryUnit::Architecture(arch) => arch.entity_name.item.item.name_utf8(),
        AnySecondaryUnit::PackageBody(pkg_body) => pkg_body.ident.tree.item.name_utf8(),
    }
}

pub fn get_design_unit_name(du: &AnyDesignUnit) -> String {
    match du {
        AnyDesignUnit::Primary(pu) => get_primary_unit_name(pu),
        AnyDesignUnit::Secondary(su) => get_secondary_unit_name(su),
    }
}

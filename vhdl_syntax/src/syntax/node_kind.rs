// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum InternalNodeKind {
    ActualPartTokens,
    SubtypeIndicationOrExpressionTokens,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum NodeKind {
    AttributeDeclaration,
    AttributeSpecification,
    AliasDeclaration,
    EntityDeclaration,
    ConfigurationDeclaration,
    ComponentDeclaration,
    Name,
    SubtypeIndication,
    Signature,
    PackageDeclaration,
    PackageInstantiationDeclaration,
    ContextDeclaration,
    ArchitectureBody,
    PackageBody,
    EntityHeader,
    DesignUnit,
    DesignFile,
    ContextClause,
    LibraryClause,
    UseClause,
    ContextReference,
    GenericClause,
    PortClause,
    InterfaceList,
    IdentifierList,
    DesignatorList,
    EntitySpecification,
    EntityNameList,
    EntityDesignator,
    Label,
    BlockStatement,
    InterfaceObjectDeclaration,
    ParenthesizedExpression,
    Expression,
    SimpleExpression,
    ExpressionList,
    Range,
    SelectedName,
    ExternalName,
    ExternalPathName,
    AttributeName,
    FunctionCallOrIndexedName,
    SliceName,
    Internal(InternalNodeKind),
    NameList,
    AssociationList,
    AssociationElement,
    FormalPart,
    ActualPart, // ...
}

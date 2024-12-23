use std::ops::BitOr;

#[allow(non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, ts_rs::TS, PartialEq)]
#[ts(export)]
#[repr(u32)]
pub enum UserPermissionEnum {
    INVENTORY_CREATE = 0b0001,
    INVENTORY_READ = 0b0010,
    INVENTORY_UPDATE = 0b0100,
    INVENTORY_DELETE = 0b1000,
    ORDER_CREATE = 0b0001_0000,
    ORDER_READ = 0b0010_0000,
    ORDER_UPDATE = 0b0100_0000,
    ORDER_DELETE = 0b1000_0000,
    CUSTOMERS_CREATE = 0b0001_0000_0000,
    CUSTOMERS_READ = 0b0010_0000_0000,
    CUSTOMERS_UPDATE = 0b0100_0000_0000,
    CUSTOMERS_DELETE = 0b1000_0000_0000,
    SUPPLIERS_CREATE = 0b0001_0000_0000_0000,
    SUPPLIERS_READ = 0b0010_0000_0000_0000,
    SUPPLIERS_UPDATE = 0b0100_0000_0000_0000,
    SUPPLIERS_DELETE = 0b1000_0000_0000_0000,
    EXPENSES_CREATE = 0b0001_0000_0000_0000_0000,
    EXPENSES_READ = 0b0010_0000_0000_0000_0000,
    EXPENSES_UPDATE = 0b0100_0000_0000_0000_0000,
    EXPENSES_DELETE = 0b1000_0000_0000_0000_0000,
    PURCHASE_CREATE = 0b0001_0000_0000_0000_0000_0000,
    PURCHASE_READ = 0b0010_0000_0000_0000_0000_0000,
    PURCHASE_UPDATE = 0b0100_0000_0000_0000_0000_0000,
    PURCHASE_DELETE = 0b1000_0000_0000_0000_0000_0000,
    PAYMENT_CREATE = 0b0001_0000_0000_0000_0000_0000_0000,
    PAYMENT_READ = 0b0010_0000_0000_0000_0000_0000_0000,
    PAYMENT_UPDATE = 0b0100_0000_0000_0000_0000_0000_0000,
    PAYMENT_DELETE = 0b1000_0000_0000_0000_0000_0000_0000,
    REPORTS = 0b0001_0000_0000_0000_0000_0000_0000_0000,
    MANAGE_DB = 0b0010_0000_0000_0000_0000_0000_0000_0000,
    SETTINGS = 0b0100_0000_0000_0000_0000_0000_0000_0000,
    ADMIN = 0xFFFF_FFFF,
}

/// Update this array when adding new permissions.
/// Used to split permissions into individual permissions.
const PERMISSION_VARIANTS: &'static [UserPermissionEnum] = &[
    UserPermissionEnum::INVENTORY_CREATE,
    UserPermissionEnum::INVENTORY_READ,
    UserPermissionEnum::INVENTORY_UPDATE,
    UserPermissionEnum::INVENTORY_DELETE,
    UserPermissionEnum::ORDER_CREATE,
    UserPermissionEnum::ORDER_READ,
    UserPermissionEnum::ORDER_UPDATE,
    UserPermissionEnum::ORDER_DELETE,
    UserPermissionEnum::CUSTOMERS_CREATE,
    UserPermissionEnum::CUSTOMERS_READ,
    UserPermissionEnum::CUSTOMERS_UPDATE,
    UserPermissionEnum::CUSTOMERS_DELETE,
    UserPermissionEnum::SUPPLIERS_CREATE,
    UserPermissionEnum::SUPPLIERS_READ,
    UserPermissionEnum::SUPPLIERS_UPDATE,
    UserPermissionEnum::SUPPLIERS_DELETE,
    UserPermissionEnum::EXPENSES_CREATE,
    UserPermissionEnum::EXPENSES_READ,
    UserPermissionEnum::EXPENSES_UPDATE,
    UserPermissionEnum::EXPENSES_DELETE,
    UserPermissionEnum::PURCHASE_CREATE,
    UserPermissionEnum::PURCHASE_READ,
    UserPermissionEnum::PURCHASE_UPDATE,
    UserPermissionEnum::PURCHASE_DELETE,
    UserPermissionEnum::PAYMENT_CREATE,
    UserPermissionEnum::PAYMENT_READ,
    UserPermissionEnum::PAYMENT_UPDATE,
    UserPermissionEnum::PAYMENT_DELETE,
    UserPermissionEnum::REPORTS,
    UserPermissionEnum::MANAGE_DB,
    UserPermissionEnum::SETTINGS,
    UserPermissionEnum::ADMIN,
];

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct UserPermissions(pub u32);

impl UserPermissions {
    pub fn has_permissions(&self, permissions: UserPermissions) -> bool {
        self.0 & permissions.0 == permissions.0
    }

    pub fn split_into_vec(&self) -> UserPermissionsVec {
        UserPermissionsVec::split_from(*self)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, ts_rs::TS)]
#[serde(transparent)]
#[ts(export)]
pub struct UserPermissionsVec(Vec<UserPermissionEnum>);

impl From<i32> for UserPermissionsVec {
    fn from(permission: i32) -> Self {
        UserPermissionsVec::from(permission as u32)
    }
}

impl UserPermissionsVec {
    /// Create a new UserPermissionsVec from a list of permissions.
    /// This will remove duplicates.
    pub fn new(permissions: Vec<UserPermissionEnum>) -> Self {
        // Flatten the permissions to remove duplicates
        UserPermissionsVec::split_from(UserPermissionsVec(permissions).flatten())
    }

    pub fn split_from(permission: UserPermissions) -> Self {
        UserPermissionsVec(
            UserPermissionEnum::variants()
                .iter()
                .filter(|p| permission.0 & **p as u32 == **p as u32)
                .copied()
                .collect(),
        )
    }

    pub fn has_permission(&self, permission: &UserPermissionEnum) -> bool {
        self.0.iter().any(|p| p.has_permission(permission))
    }

    pub fn flatten(&self) -> UserPermissions {
        self.0.iter().fold(UserPermissions(0), |acc, p| {
            UserPermissions(acc.0 | *p as u32)
        })
    }
}

impl From<UserPermissionsVec> for UserPermissions {
    fn from(permissions: UserPermissionsVec) -> Self {
        permissions.flatten()
    }
}

impl From<u32> for UserPermissions {
    fn from(permission: u32) -> Self {
        UserPermissions(permission)
    }
}

impl From<u32> for UserPermissionsVec {
    fn from(permission: u32) -> Self {
        UserPermissionsVec::split_from(UserPermissions(permission))
    }
}

impl Into<u32> for UserPermissionEnum {
    fn into(self) -> u32 {
        self as u32
    }
}

impl BitOr for UserPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl UserPermissionEnum {
    pub const fn has_permission(&self, permission: &UserPermissionEnum) -> bool {
        (*self as u32 & *permission as u32) == *permission as u32
    }

    pub const fn variants() -> &'static [UserPermissionEnum] {
        PERMISSION_VARIANTS
    }
}

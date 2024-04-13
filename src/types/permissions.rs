use std::ops::BitOr;

#[allow(non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, ts_rs::TS)]
#[ts(export)]
#[repr(u32)]
pub enum UserPermission {
    PRODUCT_READ = 0b0001,
    PRODUCT_WRITE = 0b0010,
    ORDER_READ = 0b0100,
    ORDER_WRITE = 0b1000,
    MANAGE_DB = 0b0100_0000,
    ADMIN = 0xFFFF_FFFF,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, ts_rs::TS)]
#[serde(transparent)]
#[ts(export)]
pub struct UserPermissionVec(Vec<UserPermission>);

impl UserPermissionVec {
    /// Create a new UserPermissionVec from a list of permissions.
    /// This will remove duplicates.
    pub fn new(permissions: Vec<UserPermission>) -> Self {
        // Flatten the permissions to remove duplicates
        UserPermissionVec::split_from(UserPermissionVec(permissions).flatten())
    }

    pub fn split_from(permission: UserPermission) -> Self {
        UserPermissionVec(
            UserPermission::variants()
                .iter()
                .filter(|p| permission.has_permission(p))
                .copied()
                .collect(),
        )
    }

    pub fn has_permission(&self, permission: &UserPermission) -> bool {
        self.0.iter().any(|p| p.has_permission(permission))
    }

    pub fn flatten(&self) -> UserPermission {
        self.0
            .iter()
            .fold(UserPermission::from(0), |acc, p| acc | *p)
    }
}

/// Update this array when adding new permissions.
/// Used to split permissions into individual permissions.
const PERMISSION_VARIANTS: &'static [UserPermission] = &[
    UserPermission::PRODUCT_READ,
    UserPermission::PRODUCT_WRITE,
    UserPermission::ORDER_READ,
    UserPermission::ORDER_WRITE,
    UserPermission::MANAGE_DB,
    UserPermission::ADMIN,
];

impl From<u32> for UserPermission {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl Into<u32> for UserPermission {
    fn into(self) -> u32 {
        self as u32
    }
}

impl BitOr for UserPermission {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self::from(self as u32 | rhs as u32)
    }
}

impl UserPermission {
    pub const fn has_permission(&self, permission: &UserPermission) -> bool {
        (*self as u32 & *permission as u32) == *permission as u32
    }

    pub const fn variants() -> &'static [UserPermission] {
        PERMISSION_VARIANTS
    }

    pub fn split_into_vec(&self) -> UserPermissionVec {
        UserPermissionVec::split_from(*self)
    }
}

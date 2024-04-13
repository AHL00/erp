
#[allow(non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
#[repr(transparent)]
pub struct UserPermissions(u32);

impl UserPermissions {
    pub const PRODUCT_READ: UserPermissions = UserPermissions(0b0001);
    pub const PRODUCT_WRITE: UserPermissions = UserPermissions(0b0010);
    pub const ORDER_READ: UserPermissions = UserPermissions(0b0100);
    pub const ORDER_WRITE: UserPermissions = UserPermissions(0b1000);
    pub const USER_READ: UserPermissions = UserPermissions(0b0001_0000);
    pub const USER_WRITE: UserPermissions = UserPermissions(0b0010_0000);
    pub const MANAGE_DB: UserPermissions = UserPermissions(0b0100_0000);
    pub const ADMIN: UserPermissions = UserPermissions(0xFFFF_FFFF);
}

impl From<u32> for UserPermissions {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl Into<u32> for UserPermissions {
    fn into(self) -> u32 {
        self.0
    }
}

impl UserPermissions {
    pub fn into_u32(self) -> u32 {
        self.0
    }

    pub fn has_permission(&self, permission: &UserPermissions) -> bool {
        let permission = permission.into_u32();

        (self.into_u32() & permission) == permission
    }
}
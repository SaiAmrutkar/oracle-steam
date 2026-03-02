// crates/steam-api/src/inventory.rs
// Complete ISteamInventory - Economy & Item System

use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};

pub type SteamInventoryResult_t = i32;
pub type SteamItemInstanceID_t = u64;
pub type SteamItemDef_t = i32;

lazy_static! {
    static ref INVENTORY_RESULTS: RwLock<HashMap<SteamInventoryResult_t, InventoryResult>> =
        RwLock::new(HashMap::new());
    static ref ITEM_DEFINITIONS: RwLock<HashMap<SteamItemDef_t, ItemDefinition>> =
        RwLock::new(HashMap::new());
    static ref USER_ITEMS: RwLock<Vec<ItemInstance>> = RwLock::new(Vec::new());
    static ref NEXT_RESULT_HANDLE: RwLock<i32> = RwLock::new(1);
}

#[repr(C)]
#[derive(Clone)]
pub struct SteamItemDetails_t {
    pub item_id: SteamItemInstanceID_t,
    pub definition: SteamItemDef_t,
    pub quantity: u16,
    pub flags: u16,
}

struct InventoryResult {
    handle: SteamInventoryResult_t,
    items: Vec<SteamItemDetails_t>,
    status: i32,
}

struct ItemDefinition {
    item_def: SteamItemDef_t,
    name: String,
    description: String,
    price: String,
    properties: HashMap<String, String>,
}

struct ItemInstance {
    instance_id: SteamItemInstanceID_t,
    definition: SteamItemDef_t,
    quantity: u16,
}

fn next_result_handle() -> SteamInventoryResult_t {
    let mut h = NEXT_RESULT_HANDLE.write();
    let val = *h;
    *h += 1;
    val
}

// ============================================================================
// RESULT HANDLING
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetResultStatus(
    result_handle: SteamInventoryResult_t,
) -> i32 {
    INVENTORY_RESULTS
        .read()
        .get(&result_handle)
        .map(|r| r.status)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetResultItems(
    result_handle: SteamInventoryResult_t,
    items_array: *mut SteamItemDetails_t,
    items_array_size: *mut u32,
) -> bool {
    if items_array_size.is_null() {
        return false;
    }

    let results = INVENTORY_RESULTS.read();
    if let Some(result) = results.get(&result_handle) {
        let count = result.items.len() as u32;

        unsafe {
            if items_array.is_null() {
                *items_array_size = count;
                return true;
            }

            let available = *items_array_size;
            let to_copy = count.min(available) as usize;

            for i in 0..to_copy {
                *items_array.add(i) = result.items[i].clone();
            }

            *items_array_size = to_copy as u32;
        }

        return true;
    }

    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetResultItemProperty(
    result_handle: SteamInventoryResult_t,
    item_index: u32,
    property_name: *const c_char,
    value: *mut c_char,
    value_size: *mut u32,
) -> bool {
    if property_name.is_null() || value_size.is_null() {
        return false;
    }

    unsafe {
        if let Ok(prop_name) = CStr::from_ptr(property_name).to_str() {
            let results = INVENTORY_RESULTS.read();
            if let Some(result) = results.get(&result_handle) {
                if let Some(item) = result.items.get(item_index as usize) {
                    let defs = ITEM_DEFINITIONS.read();
                    if let Some(def) = defs.get(&item.definition) {
                        if let Some(prop_value) = def.properties.get(prop_name) {
                            let bytes = prop_value.as_bytes();
                            let required_size = bytes.len() as u32 + 1;

                            if value.is_null() {
                                *value_size = required_size;
                                return true;
                            }

                            if *value_size >= required_size {
                                let len = bytes.len();
                                std::ptr::copy_nonoverlapping(
                                    bytes.as_ptr(),
                                    value as *mut u8,
                                    len,
                                );
                                *(value as *mut u8).add(len) = 0;
                                *value_size = required_size;
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetResultTimestamp(
    result_handle: SteamInventoryResult_t,
) -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_CheckResultSteamID(
    result_handle: SteamInventoryResult_t,
    steam_id_expected: u64,
) -> bool {
    INVENTORY_RESULTS.read().contains_key(&result_handle)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_DestroyResult(result_handle: SteamInventoryResult_t) {
    INVENTORY_RESULTS.write().remove(&result_handle);
}

// ============================================================================
// GETTING ITEMS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetAllItems(
    result_handle: *mut SteamInventoryResult_t,
) -> bool {
    if result_handle.is_null() {
        return false;
    }

    let handle = next_result_handle();

    let items: Vec<SteamItemDetails_t> = USER_ITEMS
        .read()
        .iter()
        .map(|item| SteamItemDetails_t {
            item_id: item.instance_id,
            definition: item.definition,
            quantity: item.quantity,
            flags: 0,
        })
        .collect();

    let result = InventoryResult {
        handle,
        items,
        status: 1, // k_EResultOK
    };

    INVENTORY_RESULTS.write().insert(handle, result);

    unsafe {
        *result_handle = handle;
    }

    println!("[Inventory] GetAllItems: handle={}", handle);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetItemsByID(
    result_handle: *mut SteamInventoryResult_t,
    instance_ids: *const SteamItemInstanceID_t,
    count: u32,
) -> bool {
    if result_handle.is_null() || instance_ids.is_null() {
        return false;
    }

    let handle = next_result_handle();

    let requested_ids: Vec<u64> =
        unsafe { std::slice::from_raw_parts(instance_ids, count as usize).to_vec() };

    let items: Vec<SteamItemDetails_t> = USER_ITEMS
        .read()
        .iter()
        .filter(|item| requested_ids.contains(&item.instance_id))
        .map(|item| SteamItemDetails_t {
            item_id: item.instance_id,
            definition: item.definition,
            quantity: item.quantity,
            flags: 0,
        })
        .collect();

    let result = InventoryResult {
        handle,
        items,
        status: 1,
    };

    INVENTORY_RESULTS.write().insert(handle, result);

    unsafe {
        *result_handle = handle;
    }

    true
}

// ============================================================================
// SERIALIZATION
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_SerializeResult(
    result_handle: SteamInventoryResult_t,
    buffer: *mut c_void,
    buffer_size: *mut u32,
) -> bool {
    if buffer_size.is_null() {
        return false;
    }

    let results = INVENTORY_RESULTS.read();
    if let Some(result) = results.get(&result_handle) {
        let serialized = serde_json::to_string(&result.items).unwrap_or_default();
        let bytes = serialized.as_bytes();
        let required_size = bytes.len() as u32;

        unsafe {
            if buffer.is_null() {
                *buffer_size = required_size;
                return true;
            }

            if *buffer_size >= required_size {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
                *buffer_size = required_size;
                return true;
            }
        }
    }

    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_DeserializeResult(
    result_handle: *mut SteamInventoryResult_t,
    buffer: *const c_void,
    buffer_size: u32,
    reserved_must_be_false: bool,
) -> bool {
    if result_handle.is_null() || buffer.is_null() {
        return false;
    }

    let data = unsafe { std::slice::from_raw_parts(buffer as *const u8, buffer_size as usize) };

    if let Ok(json_str) = std::str::from_utf8(data) {
        if let Ok(items) = serde_json::from_str::<Vec<SteamItemDetails_t>>(json_str) {
            let handle = next_result_handle();

            let result = InventoryResult {
                handle,
                items,
                status: 1,
            };

            INVENTORY_RESULTS.write().insert(handle, result);

            unsafe {
                *result_handle = handle;
            }

            return true;
        }
    }

    false
}

// ============================================================================
// GENERATING & MODIFYING ITEMS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GenerateItems(
    result_handle: *mut SteamInventoryResult_t,
    array_item_defs: *const SteamItemDef_t,
    array_quantity: *const u32,
    array_length: u32,
) -> bool {
    if result_handle.is_null() || array_item_defs.is_null() || array_quantity.is_null() {
        return false;
    }

    let handle = next_result_handle();
    let mut new_items = Vec::new();

    unsafe {
        for i in 0..array_length as usize {
            let def = *array_item_defs.add(i);
            let qty = *array_quantity.add(i);

            let instance_id = rand::random::<u64>();

            USER_ITEMS.write().push(ItemInstance {
                instance_id,
                definition: def,
                quantity: qty as u16,
            });

            new_items.push(SteamItemDetails_t {
                item_id: instance_id,
                definition: def,
                quantity: qty as u16,
                flags: 0,
            });
        }

        *result_handle = handle;
    }

    let result = InventoryResult {
        handle,
        items: new_items,
        status: 1,
    };

    INVENTORY_RESULTS.write().insert(handle, result);

    println!("[Inventory] Generated {} items", array_length);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GrantPromoItems(
    result_handle: *mut SteamInventoryResult_t,
) -> bool {
    println!("[Inventory] Granting promo items");
    SteamAPI_ISteamInventory_GetAllItems(result_handle)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_AddPromoItem(
    result_handle: *mut SteamInventoryResult_t,
    item_def: SteamItemDef_t,
) -> bool {
    SteamAPI_ISteamInventory_GenerateItems(result_handle, &item_def, &1, 1)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_AddPromoItems(
    result_handle: *mut SteamInventoryResult_t,
    array_item_defs: *const SteamItemDef_t,
    array_length: u32,
) -> bool {
    let quantities: Vec<u32> = vec![1; array_length as usize];
    SteamAPI_ISteamInventory_GenerateItems(
        result_handle,
        array_item_defs,
        quantities.as_ptr(),
        array_length,
    )
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_ConsumeItem(
    result_handle: *mut SteamInventoryResult_t,
    item_consume: SteamItemInstanceID_t,
    quantity: u32,
) -> bool {
    let mut items = USER_ITEMS.write();

    if let Some(item) = items.iter_mut().find(|i| i.instance_id == item_consume) {
        if item.quantity >= quantity as u16 {
            item.quantity -= quantity as u16;

            if item.quantity == 0 {
                items.retain(|i| i.instance_id != item_consume);
            }

            println!("[Inventory] Consumed {} of item {}", quantity, item_consume);
            drop(items);
            return SteamAPI_ISteamInventory_GetAllItems(result_handle);
        }
    }

    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_ExchangeItems(
    result_handle: *mut SteamInventoryResult_t,
    array_generate: *const SteamItemDef_t,
    array_generate_quantity: *const u32,
    array_generate_length: u32,
    array_destroy: *const SteamItemInstanceID_t,
    array_destroy_quantity: *const u32,
    array_destroy_length: u32,
) -> bool {
    // Consume items
    unsafe {
        for i in 0..array_destroy_length as usize {
            let instance_id = *array_destroy.add(i);
            let qty = *array_destroy_quantity.add(i);

            let mut items = USER_ITEMS.write();
            if let Some(item) = items.iter_mut().find(|i| i.instance_id == instance_id) {
                if item.quantity >= qty as u16 {
                    item.quantity -= qty as u16;
                    if item.quantity == 0 {
                        items.retain(|i| i.instance_id != instance_id);
                    }
                }
            }
        }
    }

    // Generate new items
    SteamAPI_ISteamInventory_GenerateItems(
        result_handle,
        array_generate,
        array_generate_quantity,
        array_generate_length,
    )
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_TransferItemQuantity(
    result_handle: *mut SteamInventoryResult_t,
    item_id_source: SteamItemInstanceID_t,
    quantity: u32,
    item_id_dest: SteamItemInstanceID_t,
) -> bool {
    let mut items = USER_ITEMS.write();

    let source_exists = items
        .iter_mut()
        .find(|i| i.instance_id == item_id_source)
        .and_then(|src| {
            if src.quantity >= quantity as u16 {
                src.quantity -= quantity as u16;
                Some(())
            } else {
                None
            }
        })
        .is_some();

    if !source_exists {
        return false;
    }

    if let Some(dest) = items.iter_mut().find(|i| i.instance_id == item_id_dest) {
        dest.quantity += quantity as u16;
    }

    drop(items);
    SteamAPI_ISteamInventory_GetAllItems(result_handle)
}

// ============================================================================
// ITEM DEFINITIONS
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_LoadItemDefinitions() -> bool {
    println!("[Inventory] Loading item definitions");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetItemDefinitionIDs(
    item_def_ids: *mut SteamItemDef_t,
    item_def_ids_array_size: *mut u32,
) -> bool {
    if item_def_ids_array_size.is_null() {
        return false;
    }

    let defs = ITEM_DEFINITIONS.read();
    let count = defs.len() as u32;

    unsafe {
        if item_def_ids.is_null() {
            *item_def_ids_array_size = count;
            return true;
        }

        let available = *item_def_ids_array_size;
        let to_copy = count.min(available);

        for (i, &def_id) in defs.keys().take(to_copy as usize).enumerate() {
            *item_def_ids.add(i) = def_id;
        }

        *item_def_ids_array_size = to_copy;
    }

    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetItemDefinitionProperty(
    item_def: SteamItemDef_t,
    property_name: *const c_char,
    value: *mut c_char,
    value_size: *mut u32,
) -> bool {
    if property_name.is_null() || value_size.is_null() {
        return false;
    }

    unsafe {
        if let Ok(prop_name) = CStr::from_ptr(property_name).to_str() {
            let defs = ITEM_DEFINITIONS.read();
            if let Some(def) = defs.get(&item_def) {
                let prop_value = match prop_name {
                    "name" => &def.name,
                    "description" => &def.description,
                    "price" => &def.price,
                    _ => {
                        if let Some(v) = def.properties.get(prop_name) {
                            v
                        } else {
                            return false;
                        }
                    }
                };

                let bytes = prop_value.as_bytes();
                let required_size = bytes.len() as u32 + 1;

                if value.is_null() {
                    *value_size = required_size;
                    return true;
                }

                if *value_size >= required_size {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), value as *mut u8, bytes.len());
                    *(value as *mut u8).add(bytes.len()) = 0;
                    *value_size = required_size;
                    return true;
                }
            }
        }
    }

    false
}

// ============================================================================
// PROMO ITEMS & PURCHASES
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_RequestEligiblePromoItemDefinitionsIDs(
    steam_id: u64,
) -> u64 {
    println!(
        "[Inventory] Requesting eligible promo items for: {}",
        steam_id
    );
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetEligiblePromoItemDefinitionIDs(
    steam_id: u64,
    item_def_ids: *mut SteamItemDef_t,
    item_def_ids_array_size: *mut u32,
) -> bool {
    if !item_def_ids_array_size.is_null() {
        unsafe {
            *item_def_ids_array_size = 0;
        }
        return true;
    }
    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_StartPurchase(
    array_item_defs: *const SteamItemDef_t,
    array_quantity: *const u32,
    array_length: u32,
) -> u64 {
    println!("[Inventory] Starting purchase of {} items", array_length);
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_RequestPrices() -> u64 {
    println!("[Inventory] Requesting prices");
    1
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetNumItemsWithPrices() -> u32 {
    ITEM_DEFINITIONS.read().len() as u32
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetItemsWithPrices(
    array_item_defs: *mut SteamItemDef_t,
    array_prices: *mut u64,
    array_base_prices: *mut u64,
    array_length: u32,
) -> bool {
    let defs = ITEM_DEFINITIONS.read();
    let count = defs.len().min(array_length as usize);

    unsafe {
        for (i, (&def_id, _def)) in defs.iter().take(count).enumerate() {
            if !array_item_defs.is_null() {
                *array_item_defs.add(i) = def_id;
            }
            if !array_prices.is_null() {
                *array_prices.add(i) = 100;
            }
            if !array_base_prices.is_null() {
                *array_base_prices.add(i) = 100;
            }
        }
    }

    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_GetItemPrice(
    item_def: SteamItemDef_t,
    price: *mut u64,
    base_price: *mut u64,
) -> bool {
    unsafe {
        if !price.is_null() {
            *price = 100;
        }
        if !base_price.is_null() {
            *base_price = 100;
        }
    }
    true
}

// ============================================================================
// PROPERTY UPDATES
// ============================================================================

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_StartUpdateProperties() -> u64 {
    let handle = next_result_handle() as u64;
    println!("[Inventory] Starting property update: {}", handle);
    handle
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_RemoveProperty(
    handle: u64,
    item_id: SteamItemInstanceID_t,
    property_name: *const c_char,
) -> bool {
    println!("[Inventory] Remove property on item: {}", item_id);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_SetProperty_String(
    handle: u64,
    item_id: SteamItemInstanceID_t,
    property_name: *const c_char,
    property_value: *const c_char,
) -> bool {
    if property_name.is_null() || property_value.is_null() {
        return false;
    }

    unsafe {
        if let (Ok(name), Ok(value)) = (
            CStr::from_ptr(property_name).to_str(),
            CStr::from_ptr(property_value).to_str(),
        ) {
            println!(
                "[Inventory] Set property {}={} on item {}",
                name, value, item_id
            );
            return true;
        }
    }

    false
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_SetProperty_Bool(
    handle: u64,
    item_id: SteamItemInstanceID_t,
    property_name: *const c_char,
    value: bool,
) -> bool {
    println!("[Inventory] Set bool property on item: {}", item_id);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_SetProperty_Int64(
    handle: u64,
    item_id: SteamItemInstanceID_t,
    property_name: *const c_char,
    value: i64,
) -> bool {
    println!("[Inventory] Set int64 property on item: {}", item_id);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_SetProperty_Float(
    handle: u64,
    item_id: SteamItemInstanceID_t,
    property_name: *const c_char,
    value: f32,
) -> bool {
    println!("[Inventory] Set float property on item: {}", item_id);
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_SubmitUpdateProperties(
    handle: u64,
    result_handle: *mut SteamInventoryResult_t,
) -> bool {
    if result_handle.is_null() {
        return false;
    }

    println!("[Inventory] Submitting property updates");
    SteamAPI_ISteamInventory_GetAllItems(result_handle)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_InspectItem(
    result_handle: *mut SteamInventoryResult_t,
    item_def: *const c_char,
) -> bool {
    println!("[Inventory] Inspecting item");
    true
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_SendItemDropHeartbeat() {
    println!("[Inventory] Item drop heartbeat");
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_TriggerItemDrop(
    result_handle: *mut SteamInventoryResult_t,
    drop_list_definition: SteamItemDef_t,
) -> bool {
    println!("[Inventory] Triggering item drop");
    SteamAPI_ISteamInventory_GetAllItems(result_handle)
}

#[no_mangle]
pub extern "C" fn SteamAPI_ISteamInventory_TradeItems(
    result_handle: *mut SteamInventoryResult_t,
    steam_id_trade_partner: u64,
    array_give: *const SteamItemInstanceID_t,
    array_give_quantity: *const u32,
    give_length: u32,
    array_get: *const SteamItemInstanceID_t,
    array_get_quantity: *const u32,
    get_length: u32,
) -> bool {
    println!("[Inventory] Trading items with: {}", steam_id_trade_partner);
    true
}

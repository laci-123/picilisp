use crate::metadata::*;
use crate::debug::*;
use crate::config;
use std::collections::{HashSet, HashMap};
use std::io::{Read, Write};
use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;



pub const CELL_SIZE_BYTES: usize = std::mem::size_of::<Cell>() + std::mem::size_of::<CellContent>();


#[derive(Default)]
pub struct Cell {
    content: Box<CellContent>,
}

impl Cell {
    pub fn new(content: MetaValue) -> Self {
        Self{ content: Box::new(CellContent::new(content)) }
    }
    
    pub fn set(&mut self, content: MetaValue) {
        self.content.as_mut().metavalue = content;
    }

    pub fn as_ptr_mut(&self) -> *mut CellContent {
        (&*self.content as *const CellContent) as *mut CellContent
    }
}


#[derive(Default)]
pub struct CellContent {
    metavalue: MetaValue,
    external_ref_count: usize,
}

impl CellContent {
    pub fn new(content: MetaValue) -> Self {
        Self{ metavalue: content, external_ref_count: 0 }
    }
}


pub struct ConsCell {
    car: *mut CellContent,
    cdr: *mut CellContent,
}

impl ConsCell {
    pub fn get_car(&self) -> GcRef {
        GcRef::new(self.car)
    }

    pub fn get_cdr(&self) -> GcRef {
        GcRef::new(self.cdr)
    }
}


#[derive(Debug)]
pub struct Symbol {
    name: Option<String>,
    own_address: *const CellContent,
}

impl Symbol {
    pub fn get_name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        }
        else {
            format!("#<symbol-{:p}>", self.own_address)
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.own_address == other.own_address
    }
}

impl Eq for Symbol {}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionKind {
    Macro,
    Lambda,
}

impl FunctionKind {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Macro  => "macro",
            Self::Lambda => "lambda",
        }
    }
}


pub struct NormalFunction {
    kind: FunctionKind,
    has_rest_params: bool,
    parameters: Vec<*mut CellContent>,
    body: *mut CellContent,
    environment: *mut CellContent,
    environment_module: String,
}

impl NormalFunction {
    pub fn get_body(&self) -> GcRef {
        GcRef::new(self.body)
    }

    pub fn non_rest_params(&self) -> ParameterIterator {
        ParameterIterator{ function: self, index: 0 }
    }

    pub fn rest_param(&self) -> Option<GcRef> {
        if self.has_rest_params {
            let p = self.parameters.last().unwrap();
            Some(GcRef::new(*p))
        }
        else {
            None
        }
    }

    pub fn get_params(&self) -> Vec<GcRef> {
        self.parameters.iter().map(|p| GcRef::new(*p)).collect()
    }

    pub fn get_kind(&self) -> FunctionKind {
        self.kind
    }

    pub fn get_env(&self) -> GcRef {
        GcRef::new(self.environment)
    }

    pub fn get_env_module(&self) -> String {
        self.environment_module.clone()
    }
}

pub struct ParameterIterator<'a> {
    function: &'a NormalFunction,
    index: usize,
}

impl<'a> Iterator for ParameterIterator<'a> {
    type Item = GcRef;

    fn next(&mut self) -> Option<GcRef> {
        let n =
        if self.function.has_rest_params {
            self.function.parameters.len() - 1
        }
        else {
            self.function.parameters.len()
        };

        if self.index < n {
            let er = GcRef::new(self.function.parameters[self.index]);
            self.index += 1;
            Some(er)
        }
        else {
            None
        }
    }
}


pub struct NativeFunction {
    kind: FunctionKind,
              // memory,     argumetns, environment, recursion depth           value  signal
    function: fn(&mut Memory, &[GcRef], GcRef,       usize)          -> Result<GcRef, GcRef>,
    parameters: Vec<String>,
}

impl NativeFunction {
    pub fn call(&self, mem: &mut Memory, args: &[GcRef], env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
        (self.function)(mem, args, env, recursion_depth)
    }

    pub fn is_the_same_as(&self, function: fn(&mut Memory, &[GcRef], GcRef, usize) -> Result<GcRef, GcRef>) -> bool {
        self.function == function
    }
}


pub enum Function {
    NormalFunction(NormalFunction),
    NativeFunction(NativeFunction),
}

impl Function {
    pub fn to_string(&self) -> String {
        let kind =
        match self.get_kind() {
            FunctionKind::Lambda => "lambda",
            FunctionKind::Macro  => "macro",
        };
        match self {
            Self::NormalFunction(nf) => format!("#<{kind}-{:?}>", nf.body),
            Self::NativeFunction(nf) => format!("#<{kind}-{:?}>", nf.function),
        }
    }
    
    #[cfg(test)]
    pub fn as_normal_function(&self) -> &NormalFunction {
        if let Self::NormalFunction(nf) = self {
            nf
        }
        else {
            panic!("attempted to cast a native function to a normal function")
        }
    }

    pub fn get_kind(&self) -> FunctionKind {
        match self {
            Self::NormalFunction(nf) => nf.kind,
            Self::NativeFunction(nf) => nf.kind,
        }
    }

    pub fn get_body(&self) -> GcRef {
        match self {
            Self::NormalFunction(nf) => nf.get_body(),
            _ => GcRef::nil(),
        }
    }

    pub fn get_env(&self) -> GcRef {
        match self {
            Self::NormalFunction(nf) => nf.get_env(),
            _ => GcRef::nil(),
        }
    }

    pub fn get_module(&self) -> String {
        match self {
            Self::NormalFunction(nf) => nf.get_env_module(),
            _ => String::new(),
        }
    }

    pub fn get_param_names(&self) -> Vec<String> {
        match self {
            Self::NormalFunction(nf) => {
                let mut param_names = vec![];
                for p in nf.non_rest_params() {
                    if let Some(PrimitiveValue::Symbol(s)) = p.get() {
                        param_names.push(s.get_name());
                    }
                    else {
                        param_names.push(format!("#<invalid-parameter-name>"));
                    }
                }
                
                if let Some(rp) = nf.rest_param() {
                    if let Some(PrimitiveValue::Symbol(s)) = rp.get() {
                        param_names.push("&".to_string());
                        param_names.push(s.get_name());
                    }
                    else {
                        param_names.push(format!("#<invalid-parameter-name>"));
                    }
                }

                param_names
            },
            Self::NativeFunction(nf) => {
                nf.parameters.clone()
            },
        }
    }
}


pub struct Trap {
    normal_body: *mut CellContent,
    trap_body: *mut CellContent,
}

impl Trap {
    pub fn to_string(&self) -> String {
        format!("#<trap-{:?}>", self.normal_body)
    }
    
    pub fn get_normal_body(&self) -> GcRef {
        GcRef::new(self.normal_body)
    }

    pub fn get_trap_body(&self) -> GcRef {
        GcRef::new(self.trap_body)
    }
}


#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TypeLabel {
    Any,
    Nil,
    Number,
    Character,
    Cons,
    List,
    String,
    Symbol,
    Function,
    Trap,
}

impl TypeLabel {
    pub fn to_string(self) -> &'static str {
        match self {
            Self::Any       => "any-type",
            Self::Nil       => "nil-type",
            Self::Number    => "number-type",
            Self::Character => "character-type",
            Self::Cons      => "conscell-type",
            Self::List      => "list-type",
            Self::String    => "string-type",
            Self::Symbol    => "symbol-type",
            Self::Function  => "function-type",
            Self::Trap      => "trap-type",
        }
    }
}


pub enum PrimitiveValue {
    Number(i64),
    Character(char),
    Cons(ConsCell),
    Symbol(Symbol),
    Function(Function),
    Trap(Trap),
}

impl PrimitiveValue { 
    #[cfg(test)]
    pub fn as_number(&self) -> &i64 {
        if let Self::Number(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-number PrimitiveValue to number")
        }
    }

    #[cfg(test)]
    pub fn as_character(&self) -> &char {
        if let Self::Character(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-character PrimitiveValue to character")
        }
    }

    pub fn as_conscell(&self) -> &ConsCell {
        if let Self::Cons(x) = self {
            x
        }
        
        else {
            panic!("attempted to cast non-conscell PrimitiveValue to conscell")
        }
    }

    pub fn as_symbol(&self) -> &Symbol{
        if let Self::Symbol(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-symbol PrimitiveValue to symbol")
        }
    }

    #[cfg(test)]
    pub fn as_function(&self) -> &Function{
        if let Self::Function(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-function PrimitiveValue to function")
        }
    }

    #[cfg(test)]
    pub fn as_trap(&self) -> &Trap{
        if let Self::Trap(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-trap PrimitiveValue to trap")
        }
    }
}


pub enum MetaValue {
    Value(PrimitiveValue),
    Meta{ value: *mut CellContent, meta: Metadata},
}

impl Default for MetaValue {
    fn default() -> Self {
        Self::Value(PrimitiveValue::Number(0))
    }
}

impl MetaValue {
    fn is_nil(&self) -> bool {
        match self {
            Self::Value(_)             => false,
            Self::Meta{value, meta: _} => value.is_null(),
        }
    }

    #[cfg(test)]
    fn is_default(&self) -> bool {
        matches!(self, Self::Value(PrimitiveValue::Number(0)))
    }

    fn get_value(&self) -> Option<&PrimitiveValue> {
        match self {
            Self::Value(v) => Some(v),
            Self::Meta{value: actual_value, meta: _} => {
                if actual_value.is_null() {
                    None
                }
                else {
                    unsafe {
                        (**actual_value).metavalue.get_value()
                    }
                }
            },
        }
    }

    fn get_meta(&self) -> Option<&Metadata> {
        match self {
            Self::Value(_) => None,
            Self::Meta{value: _, meta: metadata} => {
                Some(metadata)
            },
        }
    }
}

pub struct GcRef {
    pointer: *mut CellContent,
}

impl GcRef {
    fn new(pointer: *mut CellContent) -> Self {
        if !pointer.is_null() {
            unsafe {
                (*pointer).external_ref_count += 1;
            }
        }

        Self{ pointer }
    }

    pub fn nil() -> Self {
        Self{ pointer: std::ptr::null_mut() }
    }

    pub fn is_nil(&self) -> bool {
        if self.pointer.is_null() {
            return true;
        }

        let content =
        unsafe {
            &(*self.pointer).metavalue
        };

        content.is_nil()
    }

    pub fn get(&self) -> Option<&PrimitiveValue> {
        if self.pointer.is_null() {
            return None;
        }
        
        let content =
        unsafe {
            &(*self.pointer).metavalue
        };

        content.get_value()
    }

    pub fn get_meta(&self) -> Option<&Metadata> {
        if self.pointer.is_null() {
            return None;
        }
        
        let content =
        unsafe {
            &(*self.pointer).metavalue
        };

        content.get_meta()
    }

    pub fn get_type(&self) -> TypeLabel {
        if self.pointer.is_null() {
            return TypeLabel::Nil;
        }

        let content =
        unsafe {
            &(*self.pointer).metavalue
        };

        match content {
            MetaValue::Value(v) => match v {
                PrimitiveValue::Number(_)    => TypeLabel::Number,
                PrimitiveValue::Character(_) => TypeLabel::Character,
                PrimitiveValue::Cons(_)      => TypeLabel::Cons,
                PrimitiveValue::Symbol(_)    => TypeLabel::Symbol,
                PrimitiveValue::Function(_)  => TypeLabel::Function,
                PrimitiveValue::Trap(_)      => TypeLabel::Trap,
            },
            MetaValue::Meta{value: actual_value, meta: _} => GcRef::new(*actual_value).get_type(),
        }
    }

    pub fn clone_without_meta(&self) -> Self {
        if self.pointer.is_null() {
            return self.clone();
        }

        let content =
        unsafe {
            &(*self.pointer).metavalue
        };

        match content {
            MetaValue::Value(_) => self.clone(),
            MetaValue::Meta{value, meta: _} => Self::new(*value),
        }
    }
}

impl Clone for GcRef {
    fn clone(&self) -> Self {
        if !self.pointer.is_null() {
            unsafe {
                (*self.pointer).external_ref_count += 1;
            }
        }

        Self{ pointer: self.pointer }
    }
}

impl Drop for GcRef {
    fn drop(&mut self) {
        if !self.pointer.is_null() {
            unsafe {
                (*self.pointer).external_ref_count -= 1;
            }
        }
    }
}


struct Module {
    name: String,
    definitions: HashMap<String, GcRef>,
    exports: Option<HashSet<String>>, // None: everything is public
}

impl Module {
    fn get(&self, name: &str, current_module: &str) -> Option<GcRef> {
        if self.exports.as_ref().map(|exports| exports.contains(name)).unwrap_or(true) || current_module == self.name {
            self.definitions.get(name).map(|x| x.clone())
        }
        else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModulError {
    AmbiguousName(Vec<String>),
    GlobalNonExistentOrPrivate,
    ModuleNonExistent,
    NoSuchModule,
}


pub struct Memory {
    // Order of fields matter!
    // Fields are dropped in declaration order.
    // `modules` and `current_module` must be dropped before `cells`,
    // because on drop `GcRef` wants to access `cells`.
    modules: HashMap<String, Rc<RefCell<Module>>>,
    current_module: Rc<RefCell<Module>>,
    symbols: HashMap<String, *const CellContent>,
    cells: Vec<Cell>,
    first_free: usize,
    pub stdout: Box<dyn Write>,
    pub stdin:  Box<dyn Read>,
    pub umbilical: Option<UmbilicalLowEnd>,
}

impl Memory {
    pub fn new() -> Self {
        let default_module = Rc::new(RefCell::new(Module{ name: "default".to_string(), definitions: HashMap::new(), exports: None }));
        Self { modules:        HashMap::from([("default".to_string(), default_module.clone())]),
               current_module: default_module,
               symbols:        HashMap::new(),
               cells:          (0 .. config::INITIAL_FREE_CELLS).map(|_| Default::default()).collect(),
               first_free:     0,
               stdout:         Box::new(std::io::stdout()),
               stdin:          Box::new(std::io::stdin()),
               umbilical:      None}
    }

    pub fn set_stdout(&mut self, stdout: Box<dyn Write>) {
        self.stdout = stdout;
    }

    pub fn set_stdin(&mut self, stdin: Box<dyn Read>) {
        self.stdin = stdin;
    }

    pub fn attach_umbilical(&mut self, umbilical: UmbilicalLowEnd) {
        self.umbilical = Some(umbilical);
    }

    pub fn get_current_module(&self) -> String {
        self.current_module.borrow().name.clone()
    }

    pub fn set_current_module(&mut self, name: &str) -> Result<(), ModulError> {
        if let Some(module) = self.modules.get(name) {
            self.current_module = module.clone();
            Ok(())
        }
        else {
            Err(ModulError::NoSuchModule)
        }
    }

    pub fn define_module(&mut self, name: &str) {
        let new_module = Rc::new(RefCell::new(Module{ name: name.to_string(), definitions: HashMap::new(), exports: None }));
        self.modules.insert(name.to_string(), new_module.clone());
        self.current_module = new_module;
    }

    pub fn add_export(&mut self, name: &str) {
        let mut current_module = self.current_module.borrow_mut();
        if let Some(exports) = &mut current_module.exports {
            exports.insert(name.to_string());
        }
        else {
            let exports = HashSet::from([name.to_string()]);
            current_module.exports = Some(exports);
        }
    }

    pub fn get_module_of_global(&self, name: &str) -> Vec<String> {
        self.modules
            .iter()
            .filter(|(_, module)| module.borrow().definitions.contains_key(name))
            .map(|(module_name, _)| module_name.clone())
            .collect()
    }

    pub fn define_global(&mut self, name: &str, value: GcRef) {
        self.current_module.borrow_mut().definitions.insert(name.to_string(), value);
    }

    pub fn undefine_global(&mut self, name: &str) {
        self.current_module.borrow_mut().definitions.remove(name);
    }

    pub fn get_global(&self, name: &str, module_name: &str) -> Result<GcRef, ModulError> {
        let mut found = false;
        let mut result = GcRef::nil();
        let mut colliding_modules = Vec::new();
        for (mn, module) in self.modules.iter() {
            if let Some(value) = module.borrow().get(name, module_name) {
                colliding_modules.push(mn.clone());
                result = value;
                found = true;
            }
        }

        if found {
            if colliding_modules.len() == 1 {
                Ok(result)
            }
            else {
                Err(ModulError::AmbiguousName(colliding_modules))
            }
        }
        else {
            Err(ModulError::GlobalNonExistentOrPrivate)
        }
    }

    pub fn get_global_from_module(&self, name: &str, module_name: &str) -> Result<GcRef, ModulError> {
        if let Some(module) = self.modules.get(module_name).map(|m| m.borrow()) {
            if module.exports.as_ref().map(|exports| exports.contains(name)).unwrap_or(true) {
                module.definitions.get(name).map(|x| x.clone()).ok_or(ModulError::GlobalNonExistentOrPrivate)
            }
            else {
                Err(ModulError::GlobalNonExistentOrPrivate)
            }
        }
        else {
            Err(ModulError::ModuleNonExistent)
        }
    }

    pub fn is_global_defined(&self, name: &str) -> bool {
        self.current_module.borrow().definitions.contains_key(name)
    }

    pub fn is_global_exported(&self, name: &str) -> bool {
        self.current_module.borrow().exports.as_ref().map(|exports| exports.contains(name)).unwrap_or(true)
    }

    pub fn symbol_for(&mut self, name: &str) -> GcRef {
        if let Some(sym_ptr) = self.symbols.get(name) {
            GcRef::new(*sym_ptr as *mut CellContent)
        }
        else {
            let sym_ptr = self.allocate_internal(MetaValue::Value(PrimitiveValue::Symbol(Symbol{ name: Some(name.to_string()), own_address: std::ptr::null() })));
            if let MetaValue::Value(PrimitiveValue::Symbol(sym)) = unsafe {&mut (*sym_ptr).metavalue} {
                sym.own_address = sym_ptr;
            }
            else {
                unreachable!();
            }

            self.symbols.insert(name.to_string(), sym_ptr);

            GcRef::new(sym_ptr)
        }
    }

    pub fn unique_symbol(&mut self) -> GcRef {
        let sym_ptr = self.allocate_internal(MetaValue::Value(PrimitiveValue::Symbol(Symbol{ name: None, own_address: std::ptr::null() })));
        if let MetaValue::Value(PrimitiveValue::Symbol(sym)) = unsafe {&mut (*sym_ptr).metavalue} {
            sym.own_address = sym_ptr;
        }
        else {
            unreachable!();
        }

        GcRef::new(sym_ptr)
    }

    pub fn allocate_metadata(&mut self, x: GcRef, md: Metadata) -> GcRef {
        if let Some(_) = x.get_meta() {
            panic!("attempted to construct metadata whose data pointer is another metadata object");
        }
        let ptr = self.allocate_internal(MetaValue::Meta { value: x.pointer, meta: md });
        GcRef::new(ptr)
    }

    pub fn allocate_number(&mut self, number: i64) -> GcRef {
        let ptr = self.allocate_internal(MetaValue::Value(PrimitiveValue::Number(number)));
        GcRef::new(ptr)
    }

    pub fn allocate_character(&mut self, character: char) -> GcRef {
        let ptr = self.allocate_internal(MetaValue::Value(PrimitiveValue::Character(character)));
        GcRef::new(ptr)
    }

    pub fn allocate_cons(&mut self, car: GcRef, cdr: GcRef) -> GcRef {
        let ptr = self.allocate_internal(MetaValue::Value(PrimitiveValue::Cons(ConsCell{ car: car.pointer, cdr: cdr.pointer })));
        GcRef::new(ptr)
    }

    pub fn allocate_normal_function(&mut self, kind: FunctionKind, has_rest_params: bool, body: GcRef, params: &[GcRef], environment: GcRef, environment_module: &str) -> GcRef {
        let mut param_ptrs = vec![];
        for param in params {
            if !matches!(param.get(), Some(PrimitiveValue::Symbol(_))) {
                panic!("Function parameter is not a Symbol");
            }
            param_ptrs.push(param.pointer);
        }

        let f = PrimitiveValue::Function(Function::NormalFunction(NormalFunction{ kind,
                                                                                  has_rest_params,
                                                                                  body: body.pointer,
                                                                                  parameters: param_ptrs,
                                                                                  environment: environment.pointer,
                                                                                  environment_module: environment_module.to_string()}));
        let ptr = self.allocate_internal(MetaValue::Value(f));
        GcRef::new(ptr)
    }

    pub fn allocate_native_function(&mut self, kind: FunctionKind, parameters: Vec<String>, function: fn(&mut Self, &[GcRef], GcRef, usize) -> Result<GcRef, GcRef>) -> GcRef {
        let ptr = self.allocate_internal(MetaValue::Value(PrimitiveValue::Function(Function::NativeFunction(NativeFunction { kind, parameters, function }))));
        GcRef::new(ptr)
    }

    pub fn allocate_trap(&mut self, normal_body: GcRef, trap_body: GcRef) -> GcRef {
        let ptr = self.allocate_internal(MetaValue::Value(PrimitiveValue::Trap(Trap{ normal_body: normal_body.pointer, trap_body: trap_body.pointer })));
        GcRef::new(ptr)
    }

    fn allocate_internal(&mut self, content: MetaValue) -> *mut CellContent {
        if self.first_free > self.cells.len() - 1 {
            self.collect();
        }
 
        let pointer = 
        if self.first_free <= self.cells.len() - 1 {
            let cell = &mut self.cells[self.first_free];
            self.first_free += 1;
            cell.set(content);
            cell.as_ptr_mut()
        }
        else {
            let new_cell = Cell::new(content);
            let ptr = new_cell.as_ptr_mut();
            self.cells.push(new_cell);
            self.first_free += 1;

            let new_cells = (self.cells.len() as f32 * config::ALLOCATION_RATIO) as usize;
            for _ in 1 .. new_cells {
                // pre-allocate a bunch of cells
                // so that `collect` won't have to run
                // on the next few times `allocate_internal` is called
                self.cells.push(Default::default());
            }

            ptr
        };

        let fc = self.free_count().to_string();
        let uc = self.used_count().to_string();
        if let Some(umb) = &mut self.umbilical {
            if umb.last_memory_send.elapsed().as_millis() > 20 {
                let mut dm = DebugMessage::new();
                dm.insert("kind".to_string(), MEMORY_SAMPLE.to_string());
                dm.insert("free-cells".to_string(), fc);
                dm.insert("used-cells".to_string(), uc);
                dm.insert("serial-number".to_string(), umb.serial_number.to_string());
                umb.to_high_end.send(dm).expect("supervisor thread disappeared");
                umb.serial_number += 1;
                umb.last_memory_send = Instant::now();
            }
        }

        pointer
    }

    fn collect(&mut self) {
        // find cells that are externally reachable (the roots)
        let mut stack = vec![];

        for i in 0 .. self.first_free {
            let cell = &self.cells[i];
            if cell.content.external_ref_count > 0 {
                stack.push(cell.as_ptr_mut());
            }
        }

        // find all reachable cells
        // starting from the roots
        // (DFS)
        let mut reachable = HashSet::new();

        while let Some(cell) = stack.pop() {
            reachable.insert(cell);
            
            if cell.is_null() {
                continue;
            }

            let content = unsafe {
                &(*cell).metavalue
            };

            let value =
            match content {
                MetaValue::Value(v) => v,
                MetaValue::Meta{value: actual_value, meta: _ } => {
                    stack.push(*actual_value);
                    continue;
                },
            };

            match value {
                PrimitiveValue::Cons(cons) => {
                    if !cons.car.is_null() {
                        stack.push(cons.car);
                    }
                    if !cons.cdr.is_null() {
                        stack.push(cons.cdr);
                    }
                },
                PrimitiveValue::Trap(trap) => {
                    if !trap.normal_body.is_null() {
                        stack.push(trap.normal_body);
                    }
                    if !trap.trap_body.is_null() {
                        stack.push(trap.trap_body);
                    }
                },
                PrimitiveValue::Function(Function::NormalFunction(f)) => {
                    if !f.body.is_null() {
                        stack.push(f.body);
                    }
                    if !f.environment.is_null() {
                        stack.push(f.environment);
                    }
                    for p in f.parameters.iter() {
                        if !p.is_null() {
                            stack.push(*p);
                        }
                    }
                },
                _ =>{},
            }
        }

        // remove unreachable cells
        let mut i = 0;
        while i < self.first_free {
            let ptr = self.cells[i].as_ptr_mut();
            if reachable.contains(&ptr) {
                i += 1;
            }
            else {
                let cell = &self.cells[i];
                if let MetaValue::Value(PrimitiveValue::Symbol(s)) = &cell.content.metavalue {
                    if let Some(name) = &s.name {
                        self.symbols.remove(name);
                    }
                }
                self.cells.swap(i, self.first_free - 1);
                self.first_free -= 1;
            }
        }

        // if there are too many free cells
        // then deallocate some, but not too many
        let max_free_cells = (self.first_free as f32 * config::MAXIMUM_FREE_RATIO) as usize;

        if self.free_count() > max_free_cells {
            let used_count = self.used_count();
            let min_free_cells = (used_count as f32 * config::MINIMUM_FREE_RATIO) as usize;
            self.cells.truncate(used_count + min_free_cells + 1);
            self.first_free = used_count;
        }
    }

    pub fn used_count(&self) -> usize {
        // ------7------
        // 0 1 2 3 4 5 6 7 8 9
        //               ^
        //               first_free
        self.first_free
    }

    pub fn free_count(&self) -> usize {
        // --------10---------
        //               --3--
        // 0 1 2 3 4 5 6 7 8 9
        //               ^
        //               first_free
        self.cells.len() - self.first_free
    }
}


#[cfg(test)]
mod tests;

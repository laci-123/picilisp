#![allow(dead_code)]

use std::collections::{HashSet, HashMap};
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};
use crate::debug::Umbilical;
use crate::metadata::*;
use crate::config;



#[derive(Default)]
pub struct Cell {
    content: Box<CellContent>,
}

impl Cell {
    pub fn new(value: PrimitiveValue) -> Self {
        Self{ content: Box::new(CellContent::new(value)) }
    }
    
    pub fn set(&mut self, value: PrimitiveValue) {
        self.content.as_mut().value = value;
    }

    pub fn as_ptr(&self) -> *const CellContent {
        &*self.content as *const CellContent
    }
    
    pub fn as_ptr_mut(&self) -> *mut CellContent {
        (&*self.content as *const CellContent) as *mut CellContent
    }
}


#[derive(Default)]
pub struct CellContent {
    value: PrimitiveValue,
    external_ref_count: usize,
    metadata: Option<Metadata>,
}

impl CellContent {
    pub fn new(value: PrimitiveValue) -> Self {
        Self{ value, external_ref_count: 0, metadata: None }
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
    Syntax,
    Macro,
    Lambda,
    SpecialLambda,
}


pub struct NormalFunction {
    kind: FunctionKind,
    has_rest_params: bool,
    parameters: Vec<*mut CellContent>,
    body: *mut CellContent,
    environment: *mut CellContent,
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
    environment: *mut CellContent,
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
    pub fn as_normal_function(&self) -> &NormalFunction {
        if let Self::NormalFunction(nf) = self {
            nf
        }
        else {
            panic!("attempted to cast a native function to a normal function")
        }
    }

    pub fn as_native_function(&self) -> &NativeFunction {
        if let Self::NativeFunction(nf) = self {
            nf
        }
        else {
            panic!("attempted to cast a normal function to a native function")
        }
    }

    pub fn get_kind(&self) -> FunctionKind {
        match self {
            Self::NormalFunction(nf) => nf.kind,
            Self::NativeFunction(nf) => nf.kind,
        }
    }
}


pub struct Trap {
    normal_body: *mut CellContent,
    trap_body: *mut CellContent,
}

impl Trap {
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


#[derive(Default)]
pub enum PrimitiveValue {
    #[default]
    Nil,
    Number(i64),
    Character(char),
    Cons(ConsCell),
    Symbol(Symbol),
    Function(Function),
    Trap(Trap),
}

impl PrimitiveValue { 
    pub fn is_nil(&self) -> bool {
        if let Self::Nil = self {
            true
        }
        else {
            false
        }
    }

    pub fn as_number(&self) -> &i64 {
        if let Self::Number(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-number PrimitiveValue to number")
        }
    }

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

    pub fn as_function(&self) -> &Function{
        if let Self::Function(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-function PrimitiveValue to function")
        }
    }

    pub fn as_trap(&self) -> &Trap{
        if let Self::Trap(x) = self {
            x
        }
        else {
            panic!("attempted to cast non-trap PrimitiveValue to trap")
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

        let value =
        unsafe {
            &(*self.pointer).value
        };

        value.is_nil()
    }

    pub fn get(&self) -> Option<&PrimitiveValue> {
        if self.pointer.is_null() {
            return None;
        }
        
        let value =
        unsafe {
            &(*self.pointer).value
        };

        Some(value)
    }

    pub fn with_metadata(self, md: Metadata) -> GcRef {
        if !self.pointer.is_null() {
            unsafe {
                (*self.pointer).metadata = Some(md);
            }
        }
        self
    }

    pub fn get_metadata(&self) -> Option<&Metadata> {
        if self.pointer.is_null() {
            None
        }
        else {
            unsafe {
                (*self.pointer).metadata.as_ref()
            }
        }
    }

    pub fn get_type(&self) -> TypeLabel {
        if self.pointer.is_null() {
            return TypeLabel::Nil;
        }

        let value = unsafe {
            &(*self.pointer).value
        };

        match value {
            PrimitiveValue::Nil          => return TypeLabel::Nil,
            PrimitiveValue::Number(_)    => return TypeLabel::Number,
            PrimitiveValue::Character(_) => return TypeLabel::Character,
            PrimitiveValue::Cons(_)      => return TypeLabel::Cons,
            PrimitiveValue::Symbol(_)    => return TypeLabel::Symbol,
            PrimitiveValue::Function(_)  => return TypeLabel::Function,
            PrimitiveValue::Trap(_)      => return TypeLabel::Trap,
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


pub struct Memory {
    // Order of fields matter!
    // Fields are dropped in declaration order.
    // `globals` must be dropped before `cells`,
    // because on drop `GcRef` wants to access `cells`.
    globals: HashMap<String, GcRef>,
    symbols: HashMap<String, *const CellContent>,
    cells: Vec<Cell>,
    first_free: usize,
    pub stdout: Arc<RwLock<dyn Write>>,
    pub umbilical: Option<Umbilical>,
}

impl Memory {
    pub fn new() -> Self {
        Self { globals:    HashMap::new(),
               symbols:    HashMap::new(),
               cells:      (0 .. config::INITIAL_FREE_CELLS).map(|_| Default::default()).collect(),
               first_free: 0,
               stdout:     Arc::new(RwLock::new(std::io::stdout())),
               umbilical:  None}
    }

    pub fn set_stdout(&mut self, stdout: Arc<RwLock<dyn Write>>) {
        self.stdout = stdout;
    }

    pub fn attach_umbilical(&mut self, umbilical: Umbilical) {
        self.umbilical = Some(umbilical);
    }

    pub fn define_global(&mut self, name: &str, value: GcRef) {
        self.globals.insert(name.to_string(), value);
    }

    pub fn undefine_global(&mut self, name: &str) {
        self.globals.remove(name);
    }

    pub fn get_global(&self, name: &str) -> Option<GcRef> {
        self.globals.get(name).map(|r| r.clone())
    }

    pub fn is_global_defined(&self, name: &str) -> bool {
        self.globals.contains_key(name)
    }

    pub fn symbol_for(&mut self, name: &str) -> GcRef {
        if let Some(sym_ptr) = self.symbols.get(name) {
            GcRef::new(*sym_ptr as *mut CellContent)
        }
        else {
            let sym_ptr = self.allocate_internal(PrimitiveValue::Symbol(Symbol{ name: Some(name.to_string()), own_address: std::ptr::null() }));
            if let PrimitiveValue::Symbol(sym) = unsafe {&mut (*sym_ptr).value} {
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
        let sym_ptr = self.allocate_internal(PrimitiveValue::Symbol(Symbol{ name: None, own_address: std::ptr::null() }));
        if let PrimitiveValue::Symbol(sym) = unsafe {&mut (*sym_ptr).value} {
            sym.own_address = sym_ptr;
        }
        else {
            unreachable!();
        }

        GcRef::new(sym_ptr)
    }

    pub fn allocate_number(&mut self, number: i64) -> GcRef {
        let ptr = self.allocate_internal(PrimitiveValue::Number(number));
        GcRef::new(ptr)
    }

    pub fn allocate_character(&mut self, character: char) -> GcRef {
        let ptr = self.allocate_internal(PrimitiveValue::Character(character));
        GcRef::new(ptr)
    }

    pub fn allocate_cons(&mut self, car: GcRef, cdr: GcRef) -> GcRef {
        let ptr = self.allocate_internal(PrimitiveValue::Cons(ConsCell{ car: car.pointer, cdr: cdr.pointer }));
        GcRef::new(ptr)
    }

    pub fn allocate_normal_function(&mut self, kind: FunctionKind, has_rest_params: bool, body: GcRef, params: &[GcRef], environment: GcRef) -> GcRef {
        let mut param_ptrs = vec![];
        for param in params {
            if !matches!(param.get().unwrap_or(&PrimitiveValue::Nil), PrimitiveValue::Symbol(_)) {
                panic!("Function parameter is not a Symbol");
            }
            param_ptrs.push(param.pointer);
        }

        let ptr = self.allocate_internal(PrimitiveValue::Function(Function::NormalFunction(NormalFunction{ kind, has_rest_params, body: body.pointer, parameters: param_ptrs, environment: environment.pointer })));
        GcRef::new(ptr)
    }

    pub fn allocate_native_function(&mut self, kind: FunctionKind, function: fn(&mut Self, &[GcRef], GcRef, usize) -> Result<GcRef, GcRef>, environment: GcRef) -> GcRef {
        let ptr = self.allocate_internal(PrimitiveValue::Function(Function::NativeFunction(NativeFunction { kind, function, environment: environment.pointer })));
        GcRef::new(ptr)
    }

    pub fn allocate_trap(&mut self, normal_body: GcRef, trap_body: GcRef) -> GcRef {
        let ptr = self.allocate_internal(PrimitiveValue::Trap(Trap{ normal_body: normal_body.pointer, trap_body: trap_body.pointer }));
        GcRef::new(ptr)
    }

    fn allocate_internal(&mut self, value: PrimitiveValue) -> *mut CellContent {
        if self.first_free > self.cells.len() - 1 {
            self.collect();
        }
 
        if self.first_free <= self.cells.len() - 1 {
            let cell = &mut self.cells[self.first_free];
            self.first_free += 1;
            cell.set(value);
            cell.as_ptr_mut()
        }
        else {
            let new_cell = Cell::new(value);
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
        }
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

            let value = unsafe {
                &(*cell).value
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
                if let PrimitiveValue::Symbol(s) = &cell.content.value {
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

    fn used_count(&self) -> usize {
        // ------7------
        // 0 1 2 3 4 5 6 7 8 9
        //               ^
        //               first_free
        self.first_free
    }

    fn free_count(&self) -> usize {
        // --------10---------
        //               --3--
        // 0 1 2 3 4 5 6 7 8 9
        //               ^
        //               first_free
        self.cells.len() - self.first_free
    }

    fn dump_memory(&self) {
        let used_count = self.used_count();
        let free_count = self.free_count();
        let total_count = self.cells.len();
        let total_size_kb = (total_count * (std::mem::size_of::<Cell>() + std::mem::size_of::<CellContent>())) as f32 / 1024.0;
        
        println!("Total: {} cells ({:.2} kB)", total_count, total_size_kb);
        println!("  - used: {}", used_count);
        println!("  - free: {}", free_count);
        println!("");

        println!("Used Address        Type      Value                                    External RefCount");
        println!("---- -------        ----      -----                                    -----------------");
        for (i, c) in self.cells.iter().enumerate() {
            let string = 
            match c.content.value {
                PrimitiveValue::Nil             => format!("NIL       NIL"),
                PrimitiveValue::Number(n)       => format!("NUMBER    {n}"),
                PrimitiveValue::Character(ch)   => format!("CHARACTER {ch}"),
                PrimitiveValue::Symbol(ref s)   => format!("{}", s.name.as_ref().map_or("UNIQUE SYMBOL".to_string(), |n| format!("SYMBOL    \"{n}\""))),
                PrimitiveValue::Cons(ref cons)  => format!("CONS      car: {car:p} cdr: {cdr:p}", car = cons.car, cdr = cons.cdr),
                PrimitiveValue::Function(ref f) => {
                    match f {
                        Function::NormalFunction(nf) => format!("FUNCTION   body: {:p}", nf.body),
                        Function::NativeFunction(_)  => format!("FUNCTION   body: <native>"),
                    }
                },
                PrimitiveValue::Trap(ref t)     => format!("TRAP      normal: {:p}, trap: {:p}", t.normal_body, t.trap_body),
            };
            let used = if i < self.first_free { "[x] " } else { "[ ] " };
            let rc = c.content.external_ref_count;
            println!("{} {:p} {:<50} {}", used, c.as_ptr(), string, rc);
        }

        println!("");
    }
}


#[cfg(test)]
mod tests;

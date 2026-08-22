//! CFG-based middle IR and structural optimization for THP.

use std::collections::VecDeque;
use std::fmt;

use thp_diagnostics::Span;
use thp_hir::{
    ArgumentTarget, BoundArguments, CalledClass, Callee, ClassId, FunctionId, LocalId, LoopClause,
    LoopClauseKind, MethodSlot, PropertyId, Statement, StatementKind, Type, TypedExpr,
    TypedExprKind,
};
use thp_syntax::{BinaryOp, UnaryOp};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlockId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Register(pub u32);

#[derive(Clone, Debug)]
pub struct Module {
    pub functions: Vec<Function>,
    pub classes: Vec<thp_hir::Class>,
    pub entry: FunctionId,
}

impl Module {
    pub fn instruction_count(&self) -> usize {
        self.functions
            .iter()
            .flat_map(|function| &function.blocks)
            .map(|block| block.instructions.len() + 1)
            .sum()
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<LocalId>,
    pub local_types: Vec<Type>,
    pub return_type: Type,
    pub owner: Option<ClassId>,
    pub static_method: bool,
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    pub register_types: Vec<Type>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ExceptionHandler {
    pub protected_blocks: Vec<BlockId>,
    pub catches: Vec<CatchHandler>,
}

#[derive(Clone, Debug)]
pub struct CatchHandler {
    pub class: Option<ClassId>,
    pub local: LocalId,
    pub target: BlockId,
}

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub destination: Option<Register>,
    pub kind: InstructionKind,
    pub ty: Option<Type>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum InstructionKind {
    Constant(Constant),
    LoadLocal(LocalId),
    StoreLocal {
        local: LocalId,
        value: Register,
    },
    Unary {
        op: UnaryOp,
        operand: Register,
    },
    Binary {
        op: BinaryOp,
        left: Register,
        right: Register,
    },
    IsNull(Register),
    Vector(Vec<Register>),
    Map(Vec<(Register, Register)>),
    Index {
        collection: Register,
        index: Register,
    },
    CollectionLen(Register),
    CollectionKeyAt {
        collection: Register,
        offset: Register,
    },
    CollectionValueAt {
        collection: Register,
        offset: Register,
    },
    SetIndex {
        collection: Register,
        index: Register,
        value: Register,
    },
    Call {
        callee: Callee,
        arguments: Vec<Register>,
    },
    DirectMethod {
        callee: Callee,
        arguments: Vec<Register>,
        called_class: CalledClass,
    },
    VirtualMethod {
        receiver: Register,
        slot: MethodSlot,
        arguments: Vec<Register>,
    },
    LateStaticMethod {
        receiver: Option<Register>,
        slot: MethodSlot,
        arguments: Vec<Register>,
    },
    NewObject(ClassId),
    GetProperty {
        object: Register,
        property: PropertyId,
    },
    SetProperty {
        object: Register,
        property: PropertyId,
        value: Register,
    },
    InitializeProperty {
        object: Register,
        property: PropertyId,
        value: Register,
    },
    InstanceOf {
        value: Register,
        class: ClassId,
    },
    AddSuppressed {
        primary: Register,
        suppressed: Register,
    },
    ChainPrevious {
        replacement: Register,
        previous: Register,
    },
    RaiseUnhandledMatch(Register),
    Phi(Vec<(BlockId, Register)>),
    Print(Register),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
    String(Vec<u8>),
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Jump(BlockId),
    Branch {
        condition: Register,
        then_block: BlockId,
        else_block: BlockId,
    },
    Return(Option<Register>),
    Throw(Register),
    Unreachable,
}

pub fn lower(module: &thp_hir::Module) -> Module {
    Module {
        entry: module.entry,
        classes: module.classes.clone(),
        functions: module.functions.iter().map(lower_function).collect(),
    }
}

fn lower_function(function: &thp_hir::Function) -> Function {
    FunctionBuilder::new(function).lower()
}

struct PendingBlock {
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
}

struct FunctionBuilder<'hir> {
    hir: &'hir thp_hir::Function,
    blocks: Vec<PendingBlock>,
    current: BlockId,
    local_types: Vec<Type>,
    register_types: Vec<Type>,
    exception_handlers: Vec<ExceptionHandler>,
    cleanups: Vec<Cleanup>,
    loops: Vec<LoopTargets>,
}

#[derive(Clone)]
enum Cleanup {
    Using {
        local: LocalId,
        close: MethodSlot,
        span: Span,
    },
    Finally {
        body: Vec<Statement>,
    },
}

#[derive(Clone, Copy)]
struct LoopTargets {
    break_target: BlockId,
    continue_target: BlockId,
    cleanup_depth: usize,
}

impl<'hir> FunctionBuilder<'hir> {
    fn new(hir: &'hir thp_hir::Function) -> Self {
        Self {
            hir,
            blocks: vec![PendingBlock {
                instructions: Vec::new(),
                terminator: None,
            }],
            current: BlockId(0),
            local_types: hir.locals.iter().map(|local| local.ty.clone()).collect(),
            register_types: Vec::new(),
            exception_handlers: Vec::new(),
            cleanups: Vec::new(),
            loops: Vec::new(),
        }
    }

    fn lower(mut self) -> Function {
        self.lower_statements(&self.hir.body);
        if !self.terminated() {
            self.terminate(if self.hir.return_type == Type::Void {
                Terminator::Return(None)
            } else {
                Terminator::Unreachable
            });
        }
        let blocks = self
            .blocks
            .into_iter()
            .enumerate()
            .map(|(index, block)| BasicBlock {
                id: BlockId(u32::try_from(index).expect("block count is limited to u32::MAX")),
                instructions: block.instructions,
                terminator: block.terminator.unwrap_or(Terminator::Unreachable),
            })
            .collect();
        Function {
            id: self.hir.id,
            name: self.hir.name.clone(),
            parameters: self.hir.parameters.clone(),
            local_types: self.local_types,
            return_type: self.hir.return_type.clone(),
            owner: self.hir.owner,
            static_method: self.hir.static_method,
            blocks,
            entry: BlockId(0),
            register_types: self.register_types,
            exception_handlers: self.exception_handlers,
            span: self.hir.span,
        }
    }

    fn lower_statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            if self.terminated() {
                break;
            }
            self.lower_statement(statement);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn lower_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Assign { local, value, .. } => {
                let value = self.lower_expression(value);
                self.emit_effect(
                    InstructionKind::StoreLocal {
                        local: *local,
                        value,
                    },
                    statement.span,
                );
            }
            StatementKind::Echo(expression) => {
                let value = self.lower_expression(expression);
                self.emit_effect(InstructionKind::Print(value), expression.span);
            }
            StatementKind::Return(value) => {
                let value = value
                    .as_ref()
                    .map(|expression| self.lower_expression(expression));
                self.emit_cleanups();
                if !self.terminated() {
                    self.terminate(Terminator::Return(value));
                }
            }
            StatementKind::If {
                branches,
                otherwise,
            } => self.lower_if(branches, otherwise),
            StatementKind::While { condition, body } => self.lower_while(condition, body),
            StatementKind::For {
                initializers,
                conditions,
                updates,
                body,
            } => self.lower_for(initializers, conditions, updates, body),
            StatementKind::Foreach {
                source,
                key,
                value,
                key_type,
                value_type,
                body,
            } => self.lower_foreach(
                source,
                *key,
                *value,
                key_type,
                value_type,
                body,
                statement.span,
            ),
            StatementKind::Break | StatementKind::Continue => {
                let targets = *self
                    .loops
                    .last()
                    .expect("typed HIR validates loop transfers");
                self.emit_cleanups_to(targets.cleanup_depth);
                if !self.terminated() {
                    self.terminate(Terminator::Jump(
                        if matches!(statement.kind, StatementKind::Break) {
                            targets.break_target
                        } else {
                            targets.continue_target
                        },
                    ));
                }
            }
            StatementKind::SetProperty {
                object,
                property,
                value,
            } => {
                let object = self.lower_expression(object);
                let value = self.lower_expression(value);
                self.emit_effect(
                    InstructionKind::SetProperty {
                        object,
                        property: *property,
                        value,
                    },
                    statement.span,
                );
            }
            StatementKind::SetIndex {
                root,
                collection_types,
                indices,
                value,
            } => {
                self.lower_index_assignment(
                    *root,
                    collection_types,
                    indices,
                    value,
                    statement.span,
                );
            }
            StatementKind::Throw(value) => {
                let value = self.lower_expression(value);
                self.terminate(Terminator::Throw(value));
            }
            StatementKind::Try {
                body,
                catches,
                finally,
            } => self.lower_try(body, catches, finally.as_deref(), statement.span),
            StatementKind::Using {
                local,
                value,
                close,
                body,
            } => self.lower_using(*local, value, *close, body, statement.span),
            StatementKind::Block(body) => self.lower_statements(body),
            StatementKind::Expression(expression) => {
                self.lower_expression(expression);
            }
        }
    }

    fn lower_try(
        &mut self,
        body: &[Statement],
        catches: &[thp_hir::Catch],
        finally: Option<&[Statement]>,
        span: Span,
    ) {
        let Some(finally) = finally else {
            self.lower_try_without_finally(body, catches);
            return;
        };
        self.lower_try_with_finally(body, catches, finally, span);
    }

    fn lower_try_without_finally(&mut self, body: &[Statement], catches: &[thp_hir::Catch]) {
        let body_block = self.new_block();
        self.terminate(Terminator::Jump(body_block));
        self.switch_to(body_block);
        self.lower_statements(body);

        let continuation = self.new_block();
        if !self.terminated() {
            self.terminate(Terminator::Jump(continuation));
        }
        let protected_blocks = (body_block.0..continuation.0).map(BlockId).collect();
        let mut handlers = Vec::with_capacity(catches.len());
        for clause in catches {
            let target = self.new_block();
            handlers.push(CatchHandler {
                class: Some(clause.class),
                local: clause.local,
                target,
            });
            self.switch_to(target);
            self.lower_statements(&clause.body);
            if !self.terminated() {
                self.terminate(Terminator::Jump(continuation));
            }
        }
        self.exception_handlers.push(ExceptionHandler {
            protected_blocks,
            catches: handlers,
        });
        self.switch_to(continuation);
    }

    #[allow(clippy::too_many_lines)]
    fn lower_try_with_finally(
        &mut self,
        body: &[Statement],
        catches: &[thp_hir::Catch],
        finally: &[Statement],
        span: Span,
    ) {
        let cleanup = Cleanup::Finally {
            body: finally.to_vec(),
        };
        self.cleanups.push(cleanup);

        let body_block = self.new_block();
        self.terminate(Terminator::Jump(body_block));
        self.switch_to(body_block);
        self.lower_statements(body);

        let normal_finally = self.new_block();
        if !self.terminated() {
            self.terminate(Terminator::Jump(normal_finally));
        }
        let body_protected = (body_block.0..normal_finally.0)
            .map(BlockId)
            .collect::<Vec<_>>();
        let mut final_protected = body_protected.clone();
        let mut handlers = Vec::with_capacity(catches.len());
        for clause in catches {
            let target = self.new_block();
            handlers.push(CatchHandler {
                class: Some(clause.class),
                local: clause.local,
                target,
            });
            self.switch_to(target);
            let catch_start = target.0;
            self.lower_statements(&clause.body);
            if !self.terminated() {
                self.terminate(Terminator::Jump(normal_finally));
            }
            let catch_end =
                u32::try_from(self.blocks.len()).expect("block count is limited to u32::MAX");
            final_protected.extend((catch_start..catch_end).map(BlockId));
        }
        if !handlers.is_empty() {
            self.exception_handlers.push(ExceptionHandler {
                protected_blocks: body_protected,
                catches: handlers,
            });
        }

        self.cleanups.pop();
        self.switch_to(normal_finally);
        self.lower_statements(finally);
        let continuation = self.new_block();
        if !self.terminated() {
            self.terminate(Terminator::Jump(continuation));
        }

        let throwable = Type::Object("Throwable".to_owned());
        let pending_local = self.new_internal_local(throwable.clone());
        let exception_finally = self.new_block();
        self.switch_to(exception_finally);
        let replacement_protected_start = exception_finally.0;
        self.lower_statements(finally);
        let rethrow = self.new_block();
        let replacement_protected_end = rethrow.0;
        if !self.terminated() {
            self.terminate(Terminator::Jump(rethrow));
        }
        self.switch_to(rethrow);
        let pending = self.emit_value(
            InstructionKind::LoadLocal(pending_local),
            throwable.clone(),
            span,
        );
        self.terminate(Terminator::Throw(pending));

        let replacement_local = self.new_internal_local(throwable.clone());
        let chain = self.new_block();
        self.switch_to(chain);
        let replacement = self.emit_value(
            InstructionKind::LoadLocal(replacement_local),
            throwable.clone(),
            span,
        );
        let previous = self.emit_value(InstructionKind::LoadLocal(pending_local), throwable, span);
        self.emit_effect(
            InstructionKind::ChainPrevious {
                replacement,
                previous,
            },
            span,
        );
        self.terminate(Terminator::Throw(replacement));

        self.exception_handlers.push(ExceptionHandler {
            protected_blocks: final_protected,
            catches: vec![CatchHandler {
                class: None,
                local: pending_local,
                target: exception_finally,
            }],
        });
        let replacement_protected = (replacement_protected_start..replacement_protected_end)
            .map(BlockId)
            .collect::<Vec<_>>();
        if !replacement_protected.is_empty() {
            self.exception_handlers.push(ExceptionHandler {
                protected_blocks: replacement_protected,
                catches: vec![CatchHandler {
                    class: None,
                    local: replacement_local,
                    target: chain,
                }],
            });
        }
        self.switch_to(continuation);
    }

    fn lower_using(
        &mut self,
        local: LocalId,
        value: &TypedExpr,
        close: MethodSlot,
        body: &[Statement],
        span: Span,
    ) {
        let value = self.lower_expression(value);
        self.emit_effect(InstructionKind::StoreLocal { local, value }, span);
        let cleanup = Cleanup::Using { local, close, span };
        self.cleanups.push(cleanup.clone());

        let body_block = self.new_block();
        self.terminate(Terminator::Jump(body_block));
        self.switch_to(body_block);
        self.lower_statements(body);

        let cleanup_block = self.new_block();
        if !self.terminated() {
            self.terminate(Terminator::Jump(cleanup_block));
        }
        let protected_blocks = (body_block.0..cleanup_block.0).map(BlockId).collect();
        self.cleanups.pop();

        self.switch_to(cleanup_block);
        self.emit_cleanup(&cleanup);
        let continuation = self.new_block();
        self.terminate(Terminator::Jump(continuation));

        let exception_local = LocalId(
            u32::try_from(self.local_types.len()).expect("local count is limited to u32::MAX"),
        );
        let throwable = Type::Object("Throwable".to_owned());
        self.local_types.push(throwable.clone());
        let handler_block = self.new_block();
        self.switch_to(handler_block);
        self.emit_cleanup(&cleanup);
        let rethrow_block = self.new_block();
        self.terminate(Terminator::Jump(rethrow_block));

        self.switch_to(rethrow_block);
        let exception = self.emit_value(
            InstructionKind::LoadLocal(exception_local),
            throwable.clone(),
            span,
        );
        self.terminate(Terminator::Throw(exception));

        let suppressed_local = LocalId(
            u32::try_from(self.local_types.len()).expect("local count is limited to u32::MAX"),
        );
        self.local_types.push(throwable.clone());
        let suppression_block = self.new_block();
        self.switch_to(suppression_block);
        let primary = self.emit_value(
            InstructionKind::LoadLocal(exception_local),
            throwable.clone(),
            span,
        );
        let suppressed = self.emit_value(
            InstructionKind::LoadLocal(suppressed_local),
            throwable,
            span,
        );
        self.emit_effect(
            InstructionKind::AddSuppressed {
                primary,
                suppressed,
            },
            span,
        );
        self.terminate(Terminator::Throw(primary));

        self.exception_handlers.push(ExceptionHandler {
            protected_blocks,
            catches: vec![CatchHandler {
                class: None,
                local: exception_local,
                target: handler_block,
            }],
        });
        self.exception_handlers.push(ExceptionHandler {
            protected_blocks: vec![handler_block],
            catches: vec![CatchHandler {
                class: None,
                local: suppressed_local,
                target: suppression_block,
            }],
        });
        self.switch_to(continuation);
    }

    fn emit_cleanups(&mut self) {
        self.emit_cleanups_to(0);
    }

    fn emit_cleanups_to(&mut self, depth: usize) {
        let cleanups = self.cleanups.clone();
        for index in (depth..cleanups.len()).rev() {
            self.cleanups.truncate(index);
            self.emit_cleanup(&cleanups[index]);
            if self.terminated() {
                break;
            }
        }
        self.cleanups = cleanups;
    }

    fn emit_cleanup(&mut self, cleanup: &Cleanup) {
        match cleanup {
            Cleanup::Using { local, close, span } => {
                let ty = self.local_types[local.0 as usize].clone();
                let receiver = self.emit_value(InstructionKind::LoadLocal(*local), ty, *span);
                self.emit_value(
                    InstructionKind::VirtualMethod {
                        receiver,
                        slot: *close,
                        arguments: Vec::new(),
                    },
                    Type::Void,
                    *span,
                );
            }
            Cleanup::Finally { body } => self.lower_statements(body),
        }
    }

    fn new_internal_local(&mut self, ty: Type) -> LocalId {
        let local = LocalId(
            u32::try_from(self.local_types.len()).expect("local count is limited to u32::MAX"),
        );
        self.local_types.push(ty);
        local
    }

    fn lower_if(&mut self, branches: &[(TypedExpr, Vec<Statement>)], otherwise: &[Statement]) {
        let end = self.new_block();
        let mut condition_block = self.current;
        for (index, (condition, body)) in branches.iter().enumerate() {
            self.switch_to(condition_block);
            let condition = self.lower_expression(condition);
            let body_block = self.new_block();
            let has_next = index + 1 < branches.len() || !otherwise.is_empty();
            let next = if has_next { self.new_block() } else { end };
            self.terminate(Terminator::Branch {
                condition,
                then_block: body_block,
                else_block: next,
            });

            self.switch_to(body_block);
            self.lower_statements(body);
            if !self.terminated() {
                self.terminate(Terminator::Jump(end));
            }
            condition_block = next;
        }
        if !otherwise.is_empty() {
            self.switch_to(condition_block);
            self.lower_statements(otherwise);
            if !self.terminated() {
                self.terminate(Terminator::Jump(end));
            }
        }
        self.switch_to(end);
    }

    fn lower_while(&mut self, condition: &TypedExpr, body: &[Statement]) {
        let condition_block = self.new_block();
        let body_block = self.new_block();
        let end = self.new_block();
        self.terminate(Terminator::Jump(condition_block));
        self.switch_to(condition_block);
        let condition = self.lower_expression(condition);
        self.terminate(Terminator::Branch {
            condition,
            then_block: body_block,
            else_block: end,
        });
        self.switch_to(body_block);
        self.loops.push(LoopTargets {
            break_target: end,
            continue_target: condition_block,
            cleanup_depth: self.cleanups.len(),
        });
        self.lower_statements(body);
        self.loops.pop();
        if !self.terminated() {
            self.terminate(Terminator::Jump(condition_block));
        }
        self.switch_to(end);
    }

    fn lower_for(
        &mut self,
        initializers: &[LoopClause],
        conditions: &[LoopClause],
        updates: &[LoopClause],
        body: &[Statement],
    ) {
        for clause in initializers {
            self.lower_loop_clause(clause);
        }
        let condition_block = self.new_block();
        let body_block = self.new_block();
        let update_block = self.new_block();
        let end = self.new_block();
        self.terminate(Terminator::Jump(condition_block));

        self.switch_to(condition_block);
        let condition = if conditions.is_empty() {
            self.emit_value(
                InstructionKind::Constant(Constant::Bool(true)),
                Type::Bool,
                self.hir.span,
            )
        } else {
            let mut result = None;
            for clause in conditions {
                result = Some(self.lower_loop_clause(clause));
            }
            result.expect("non-empty conditions produce a value")
        };
        self.terminate(Terminator::Branch {
            condition,
            then_block: body_block,
            else_block: end,
        });

        self.switch_to(body_block);
        self.loops.push(LoopTargets {
            break_target: end,
            continue_target: update_block,
            cleanup_depth: self.cleanups.len(),
        });
        self.lower_statements(body);
        self.loops.pop();
        if !self.terminated() {
            self.terminate(Terminator::Jump(update_block));
        }

        self.switch_to(update_block);
        for clause in updates {
            self.lower_loop_clause(clause);
        }
        if !self.terminated() {
            self.terminate(Terminator::Jump(condition_block));
        }
        self.switch_to(end);
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn lower_foreach(
        &mut self,
        source: &TypedExpr,
        key: Option<LocalId>,
        value: LocalId,
        key_type: &Type,
        value_type: &Type,
        body: &[Statement],
        span: Span,
    ) {
        let source = self.lower_expression(source);
        let length = self.emit_value(InstructionKind::CollectionLen(source), Type::Int, span);
        let offset_local = LocalId(
            u32::try_from(self.local_types.len()).expect("local count is limited to u32::MAX"),
        );
        self.local_types.push(Type::Int);
        let zero = self.emit_value(
            InstructionKind::Constant(Constant::Integer(0)),
            Type::Int,
            span,
        );
        self.emit_effect(
            InstructionKind::StoreLocal {
                local: offset_local,
                value: zero,
            },
            span,
        );

        let condition_block = self.new_block();
        let body_block = self.new_block();
        let increment_block = self.new_block();
        let end = self.new_block();
        self.terminate(Terminator::Jump(condition_block));

        self.switch_to(condition_block);
        let offset = self.emit_value(InstructionKind::LoadLocal(offset_local), Type::Int, span);
        let condition = self.emit_value(
            InstructionKind::Binary {
                op: BinaryOp::Less,
                left: offset,
                right: length,
            },
            Type::Bool,
            span,
        );
        self.terminate(Terminator::Branch {
            condition,
            then_block: body_block,
            else_block: end,
        });

        self.switch_to(body_block);
        let offset = self.emit_value(InstructionKind::LoadLocal(offset_local), Type::Int, span);
        if let Some(key) = key {
            let entry_key = self.emit_value(
                InstructionKind::CollectionKeyAt {
                    collection: source,
                    offset,
                },
                key_type.clone(),
                span,
            );
            self.emit_effect(
                InstructionKind::StoreLocal {
                    local: key,
                    value: entry_key,
                },
                span,
            );
        }
        let entry_value = self.emit_value(
            InstructionKind::CollectionValueAt {
                collection: source,
                offset,
            },
            value_type.clone(),
            span,
        );
        self.emit_effect(
            InstructionKind::StoreLocal {
                local: value,
                value: entry_value,
            },
            span,
        );
        self.loops.push(LoopTargets {
            break_target: end,
            continue_target: increment_block,
            cleanup_depth: self.cleanups.len(),
        });
        self.lower_statements(body);
        self.loops.pop();
        if !self.terminated() {
            self.terminate(Terminator::Jump(increment_block));
        }

        self.switch_to(increment_block);
        let offset = self.emit_value(InstructionKind::LoadLocal(offset_local), Type::Int, span);
        let one = self.emit_value(
            InstructionKind::Constant(Constant::Integer(1)),
            Type::Int,
            span,
        );
        let next = self.emit_value(
            InstructionKind::Binary {
                op: BinaryOp::Add,
                left: offset,
                right: one,
            },
            Type::Int,
            span,
        );
        self.emit_effect(
            InstructionKind::StoreLocal {
                local: offset_local,
                value: next,
            },
            span,
        );
        self.terminate(Terminator::Jump(condition_block));
        self.switch_to(end);
    }

    fn lower_loop_clause(&mut self, clause: &LoopClause) -> Register {
        match &clause.kind {
            LoopClauseKind::Assign { local, value } => {
                let value = self.lower_expression(value);
                self.emit_effect(
                    InstructionKind::StoreLocal {
                        local: *local,
                        value,
                    },
                    clause.span,
                );
                value
            }
            LoopClauseKind::SetProperty {
                object,
                property,
                value,
            } => {
                let object = self.lower_expression(object);
                let value = self.lower_expression(value);
                self.emit_effect(
                    InstructionKind::SetProperty {
                        object,
                        property: *property,
                        value,
                    },
                    clause.span,
                );
                value
            }
            LoopClauseKind::SetIndex {
                root,
                collection_types,
                indices,
                value,
            } => self.lower_index_assignment(*root, collection_types, indices, value, clause.span),
            LoopClauseKind::Expression(expression) => self.lower_expression(expression),
        }
    }

    fn lower_index_assignment(
        &mut self,
        root: LocalId,
        collection_types: &[Type],
        indices: &[TypedExpr],
        value: &TypedExpr,
        span: Span,
    ) -> Register {
        let root_type = self.local_types[root.0 as usize].clone();
        let root_value = self.emit_value(InstructionKind::LoadLocal(root), root_type, span);
        let mut collections = vec![root_value];
        let mut lowered_indices = Vec::with_capacity(indices.len());
        for (position, index) in indices.iter().enumerate() {
            let index = self.lower_expression(index);
            lowered_indices.push(index);
            if position + 1 < indices.len() {
                let collection = self.emit_value(
                    InstructionKind::Index {
                        collection: collections[position],
                        index,
                    },
                    collection_types[position + 1].clone(),
                    indices[position].span,
                );
                collections.push(collection);
            }
        }
        let mut updated = self.lower_expression(value);
        for position in (0..indices.len()).rev() {
            updated = self.emit_value(
                InstructionKind::SetIndex {
                    collection: collections[position],
                    index: lowered_indices[position],
                    value: updated,
                },
                collection_types[position].clone(),
                span,
            );
        }
        self.emit_effect(
            InstructionKind::StoreLocal {
                local: root,
                value: updated,
            },
            span,
        );
        updated
    }

    #[allow(clippy::too_many_lines)]
    fn lower_expression(&mut self, expression: &TypedExpr) -> Register {
        match &expression.kind {
            TypedExprKind::Binary {
                op: BinaryOp::And,
                left,
                right,
            } => self.lower_short_circuit(left, right, false, expression.span),
            TypedExprKind::Binary {
                op: BinaryOp::Or,
                left,
                right,
            } => self.lower_short_circuit(left, right, true, expression.span),
            TypedExprKind::Binary {
                op: BinaryOp::Coalesce,
                left,
                right,
            } => self.lower_coalesce(left, right, expression),
            kind => {
                let instruction = match kind {
                    TypedExprKind::Integer(value) => {
                        InstructionKind::Constant(Constant::Integer(*value))
                    }
                    TypedExprKind::Float(value) => {
                        InstructionKind::Constant(Constant::Float(*value))
                    }
                    TypedExprKind::Bool(value) => InstructionKind::Constant(Constant::Bool(*value)),
                    TypedExprKind::Null => InstructionKind::Constant(Constant::Null),
                    TypedExprKind::String(value) => {
                        InstructionKind::Constant(Constant::String(value.clone()))
                    }
                    TypedExprKind::Local(local) => InstructionKind::LoadLocal(*local),
                    TypedExprKind::Vector(values) => InstructionKind::Vector(
                        values
                            .iter()
                            .map(|value| self.lower_expression(value))
                            .collect(),
                    ),
                    TypedExprKind::Map(entries) => InstructionKind::Map(
                        entries
                            .iter()
                            .map(|(key, value)| {
                                (self.lower_expression(key), self.lower_expression(value))
                            })
                            .collect(),
                    ),
                    TypedExprKind::Unary { op, operand } => InstructionKind::Unary {
                        op: *op,
                        operand: self.lower_expression(operand),
                    },
                    TypedExprKind::Binary { op, left, right } => InstructionKind::Binary {
                        op: *op,
                        left: self.lower_expression(left),
                        right: self.lower_expression(right),
                    },
                    TypedExprKind::Call { callee, arguments } => InstructionKind::Call {
                        callee: *callee,
                        arguments: self.lower_bound_arguments(arguments),
                    },
                    TypedExprKind::DirectMethod {
                        callee,
                        receiver,
                        arguments,
                        called_class,
                    } => {
                        let mut arguments = self.lower_bound_arguments(arguments);
                        if let Some(receiver) = receiver {
                            arguments.insert(0, self.lower_expression(receiver));
                        }
                        InstructionKind::DirectMethod {
                            callee: *callee,
                            arguments,
                            called_class: *called_class,
                        }
                    }
                    TypedExprKind::VirtualMethod {
                        receiver,
                        slot,
                        arguments,
                    } => InstructionKind::VirtualMethod {
                        receiver: self.lower_expression(receiver),
                        slot: *slot,
                        arguments: self.lower_bound_arguments(arguments),
                    },
                    TypedExprKind::LateStaticMethod {
                        receiver,
                        slot,
                        arguments,
                    } => InstructionKind::LateStaticMethod {
                        receiver: receiver
                            .as_ref()
                            .map(|receiver| self.lower_expression(receiver)),
                        slot: *slot,
                        arguments: self.lower_bound_arguments(arguments),
                    },
                    TypedExprKind::Index { collection, index } => InstructionKind::Index {
                        collection: self.lower_expression(collection),
                        index: self.lower_expression(index),
                    },
                    TypedExprKind::New {
                        class,
                        constructor,
                        initializers,
                        arguments,
                    } => {
                        let object = self.emit_value(
                            InstructionKind::NewObject(*class),
                            expression.ty.clone(),
                            expression.span,
                        );
                        for (property, initializer) in initializers {
                            let value = self.lower_expression(initializer);
                            self.emit_effect(
                                InstructionKind::InitializeProperty {
                                    object,
                                    property: *property,
                                    value,
                                },
                                initializer.span,
                            );
                        }
                        if let Some(constructor) = constructor {
                            let mut arguments = self.lower_bound_arguments(arguments);
                            arguments.insert(0, object);
                            self.emit_value(
                                InstructionKind::DirectMethod {
                                    callee: *constructor,
                                    arguments,
                                    called_class: CalledClass::Explicit(*class),
                                },
                                Type::Void,
                                expression.span,
                            );
                        }
                        return object;
                    }
                    TypedExprKind::Property { object, property } => InstructionKind::GetProperty {
                        object: self.lower_expression(object),
                        property: *property,
                    },
                    TypedExprKind::InstanceOf { value, class } => InstructionKind::InstanceOf {
                        value: self.lower_expression(value),
                        class: *class,
                    },
                    TypedExprKind::Match { .. } => {
                        return self.lower_match(expression);
                    }
                };
                self.emit_value(instruction, expression.ty.clone(), expression.span)
            }
        }
    }

    fn lower_bound_arguments(&mut self, arguments: &BoundArguments) -> Vec<Register> {
        let mut parameters = vec![None; arguments.parameter_count];
        let mut variadic = Vec::new();
        for argument in &arguments.explicit {
            let value = self.lower_expression(&argument.value);
            match argument.target {
                ArgumentTarget::Parameter(index) => parameters[index] = Some(value),
                ArgumentTarget::Variadic => variadic.push(value),
            }
        }
        for argument in &arguments.defaults {
            let value = self.lower_expression(&argument.value);
            let ArgumentTarget::Parameter(index) = argument.target else {
                unreachable!("defaults never target a variadic parameter")
            };
            parameters[index] = Some(value);
        }
        if let Some(index) = arguments.variadic_parameter {
            let element = arguments
                .variadic_type
                .clone()
                .expect("variadic arguments carry an element type");
            parameters[index] = Some(self.emit_value(
                InstructionKind::Vector(variadic),
                Type::Vector(Box::new(element)),
                self.hir.span,
            ));
        }
        parameters
            .into_iter()
            .map(|parameter| parameter.expect("typed call binds every parameter"))
            .collect()
    }

    fn lower_match(&mut self, expression: &TypedExpr) -> Register {
        let TypedExprKind::Match { subject, arms } = &expression.kind else {
            unreachable!("match lowering requires a match expression")
        };
        let subject = self.lower_expression(subject);
        let merge = self.new_block();
        let default = arms.iter().find(|arm| arm.default);
        let mut inputs = Vec::new();

        for arm in arms.iter().filter(|arm| !arm.default) {
            let value_block = self.new_block();
            for condition in &arm.conditions {
                let condition_span = condition.span;
                let condition = self.lower_expression(condition);
                let matches = self.emit_value(
                    InstructionKind::Binary {
                        op: BinaryOp::StrictEqual,
                        left: subject,
                        right: condition,
                    },
                    Type::Bool,
                    condition_span,
                );
                let next = self.new_block();
                self.terminate(Terminator::Branch {
                    condition: matches,
                    then_block: value_block,
                    else_block: next,
                });
                self.switch_to(next);
            }
            let next_arm = self.current;
            self.switch_to(value_block);
            let value = self.lower_expression(&arm.value);
            let predecessor = self.current;
            if !self.terminated() {
                self.terminate(Terminator::Jump(merge));
                inputs.push((predecessor, value));
            }
            self.switch_to(next_arm);
        }

        if let Some(default) = default {
            let value = self.lower_expression(&default.value);
            let predecessor = self.current;
            if !self.terminated() {
                self.terminate(Terminator::Jump(merge));
                inputs.push((predecessor, value));
            }
        } else if inputs.is_empty() {
            let never = self.emit_value(
                InstructionKind::RaiseUnhandledMatch(subject),
                expression.ty.clone(),
                expression.span,
            );
            self.terminate(Terminator::Unreachable);
            return never;
        } else {
            self.emit_effect(
                InstructionKind::RaiseUnhandledMatch(subject),
                expression.span,
            );
            self.terminate(Terminator::Unreachable);
        }

        self.switch_to(merge);
        self.emit_value(
            InstructionKind::Phi(inputs),
            expression.ty.clone(),
            expression.span,
        )
    }

    fn lower_short_circuit(
        &mut self,
        left: &TypedExpr,
        right: &TypedExpr,
        short_value: bool,
        span: Span,
    ) -> Register {
        let left = self.lower_expression(left);
        let short_block = self.new_block();
        let right_block = self.new_block();
        let merge = self.new_block();
        let (then_block, else_block) = if short_value {
            (short_block, right_block)
        } else {
            (right_block, short_block)
        };
        self.terminate(Terminator::Branch {
            condition: left,
            then_block,
            else_block,
        });
        self.switch_to(short_block);
        let short = self.emit_value(
            InstructionKind::Constant(Constant::Bool(short_value)),
            Type::Bool,
            span,
        );
        self.terminate(Terminator::Jump(merge));
        self.switch_to(right_block);
        let right = self.lower_expression(right);
        self.terminate(Terminator::Jump(merge));
        self.switch_to(merge);
        self.emit_value(
            InstructionKind::Phi(vec![(short_block, short), (right_block, right)]),
            Type::Bool,
            span,
        )
    }

    fn lower_coalesce(
        &mut self,
        left: &TypedExpr,
        right: &TypedExpr,
        expression: &TypedExpr,
    ) -> Register {
        let left = self.lower_expression(left);
        let is_null = self.emit_value(
            InstructionKind::IsNull(left),
            Type::Bool,
            left_span(left, self),
        );
        let right_block = self.new_block();
        let left_block = self.new_block();
        let merge = self.new_block();
        self.terminate(Terminator::Branch {
            condition: is_null,
            then_block: right_block,
            else_block: left_block,
        });
        self.switch_to(right_block);
        let right = self.lower_expression(right);
        self.terminate(Terminator::Jump(merge));
        self.switch_to(left_block);
        self.terminate(Terminator::Jump(merge));
        self.switch_to(merge);
        self.emit_value(
            InstructionKind::Phi(vec![(right_block, right), (left_block, left)]),
            expression.ty.clone(),
            expression.span,
        )
    }

    fn emit_value(&mut self, kind: InstructionKind, ty: Type, span: Span) -> Register {
        let register = Register(
            u32::try_from(self.register_types.len())
                .expect("register count is limited to u32::MAX"),
        );
        self.register_types.push(ty.clone());
        self.current_block_mut().instructions.push(Instruction {
            destination: Some(register),
            kind,
            ty: Some(ty),
            span,
        });
        register
    }

    fn emit_effect(&mut self, kind: InstructionKind, span: Span) {
        self.current_block_mut().instructions.push(Instruction {
            destination: None,
            kind,
            ty: None,
            span,
        });
    }

    fn new_block(&mut self) -> BlockId {
        let id =
            BlockId(u32::try_from(self.blocks.len()).expect("block count is limited to u32::MAX"));
        self.blocks.push(PendingBlock {
            instructions: Vec::new(),
            terminator: None,
        });
        id
    }

    fn switch_to(&mut self, block: BlockId) {
        self.current = block;
    }

    fn terminate(&mut self, terminator: Terminator) {
        assert!(
            self.current_block_mut()
                .terminator
                .replace(terminator)
                .is_none(),
            "MIR block terminated twice"
        );
    }

    fn terminated(&self) -> bool {
        self.blocks[self.current.0 as usize].terminator.is_some()
    }

    fn current_block_mut(&mut self) -> &mut PendingBlock {
        &mut self.blocks[self.current.0 as usize]
    }
}

fn left_span(register: Register, builder: &FunctionBuilder<'_>) -> Span {
    builder
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|instruction| instruction.destination == Some(register))
        .map_or(builder.hir.span, |instruction| instruction.span)
}

/// Removes blocks not reachable from the entry and rewrites block references.
///
/// # Panics
///
/// Panics when a malformed function contains more than `u32::MAX` blocks.
pub fn eliminate_unreachable(function: &mut Function) {
    let mut reachable = vec![false; function.blocks.len()];
    let mut queue = VecDeque::from([function.entry]);
    queue.extend(
        function
            .exception_handlers
            .iter()
            .flat_map(|handler| handler.catches.iter().map(|clause| clause.target)),
    );
    while let Some(block) = queue.pop_front() {
        let index = block.0 as usize;
        if reachable[index] {
            continue;
        }
        reachable[index] = true;
        match function.blocks[index].terminator {
            Terminator::Jump(target) => queue.push_back(target),
            Terminator::Branch {
                then_block,
                else_block,
                ..
            } => {
                queue.push_back(then_block);
                queue.push_back(else_block);
            }
            Terminator::Return(_) | Terminator::Throw(_) | Terminator::Unreachable => {}
        }
    }
    if reachable.iter().all(|value| *value) {
        return;
    }
    let mut remap = vec![BlockId(u32::MAX); function.blocks.len()];
    let mut blocks = Vec::new();
    for (old, mut block) in function.blocks.drain(..).enumerate() {
        if reachable[old] {
            let new =
                BlockId(u32::try_from(blocks.len()).expect("block count is limited to u32::MAX"));
            remap[old] = new;
            block.id = new;
            blocks.push(block);
        }
    }
    for block in &mut blocks {
        for instruction in &mut block.instructions {
            if let InstructionKind::Phi(inputs) = &mut instruction.kind {
                inputs.retain(|(predecessor, _)| reachable[predecessor.0 as usize]);
                for (predecessor, _) in inputs {
                    *predecessor = remap[predecessor.0 as usize];
                }
            }
        }
        match &mut block.terminator {
            Terminator::Jump(target) => *target = remap[target.0 as usize],
            Terminator::Branch {
                then_block,
                else_block,
                ..
            } => {
                *then_block = remap[then_block.0 as usize];
                *else_block = remap[else_block.0 as usize];
            }
            Terminator::Return(_) | Terminator::Throw(_) | Terminator::Unreachable => {}
        }
    }
    for handler in &mut function.exception_handlers {
        handler
            .protected_blocks
            .retain(|block| reachable[block.0 as usize]);
        for block in &mut handler.protected_blocks {
            *block = remap[block.0 as usize];
        }
        for clause in &mut handler.catches {
            clause.target = remap[clause.target.0 as usize];
        }
    }
    function.entry = remap[function.entry.0 as usize];
    function.blocks = blocks;
}

impl fmt::Display for Module {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for function in &self.functions {
            writeln!(
                formatter,
                "fn {}#{} (locals={}, registers={}) -> {} {{",
                function.name,
                function.id.0,
                function.local_types.len(),
                function.register_types.len(),
                function.return_type
            )?;
            for block in &function.blocks {
                writeln!(formatter, "  bb{}:", block.id.0)?;
                for instruction in &block.instructions {
                    if let Some(destination) = instruction.destination {
                        write!(formatter, "    r{} = ", destination.0)?;
                    } else {
                        formatter.write_str("    ")?;
                    }
                    writeln!(formatter, "{:?}", instruction.kind)?;
                }
                writeln!(formatter, "    -> {:?}", block.terminator)?;
            }
            writeln!(formatter, "}}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use thp_diagnostics::SourceFile;
    use thp_hir::lower as lower_hir;
    use thp_syntax::parse;

    use super::{InstructionKind, Terminator, lower};

    fn compile(source: &str) -> super::Module {
        let source = SourceFile::new("test.thp", source);
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let hir = lower_hir(&parsed.program);
        assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);
        lower(&hir.module)
    }

    #[test]
    fn lowers_branches_and_loops_to_cfg() {
        let module = compile(
            r#"<?thp
$value: int = 0;
while ($value < 2) {
    $value = $value + 1;
}
if ($value === 2) { echo "yes"; } else { echo "no"; }
"#,
        );
        let main = &module.functions[0];
        assert!(main.blocks.len() >= 7);
        assert!(
            main.blocks
                .iter()
                .any(|block| matches!(block.terminator, Terminator::Branch { .. }))
        );
    }

    #[test]
    fn short_circuit_uses_phi() {
        let module = compile("<?thp\n$value: bool = false && true;");
        assert!(module.functions[0].blocks.iter().any(|block| {
            block
                .instructions
                .iter()
                .any(|instruction| matches!(instruction.kind, InstructionKind::Phi(_)))
        }));
    }

    #[test]
    fn lowers_collection_control_flow_and_match_to_vm_primitives() {
        let module = compile(
            r"<?thp
$values: vector<int> = [1, 2];
foreach ($values as $index => $value) {
    if ($index === 0) { continue; }
    $values[$index] = match ($value) { 2 => 3, default => 0 };
    break;
}
",
        );
        let instructions = module.functions[0]
            .blocks
            .iter()
            .flat_map(|block| &block.instructions)
            .map(|instruction| &instruction.kind)
            .collect::<Vec<_>>();
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, InstructionKind::CollectionLen(_)))
        );
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, InstructionKind::CollectionKeyAt { .. }))
        );
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, InstructionKind::SetIndex { .. }))
        );
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, InstructionKind::Phi(_)))
        );
    }

    #[test]
    fn distinguishes_direct_virtual_and_late_static_dispatch() {
        let module = compile(
            r#"<?thp
class Base {
    public function virtualValue(): string { return "base"; }
    public final function finalValue(): string { return "final"; }
    public static function kind(): string { return "base"; }
    public static function forwardedKind(): string { return static::kind(); }
}
class Child extends Base {
    public function virtualValue(): string { return "child"; }
    public static function kind(): string { return "child"; }
    public function lexical(): string { return self::finalValue(); }
    public function parentValue(): string { return parent::virtualValue(); }
}
$child = new Child();
echo $child->virtualValue();
echo $child->finalValue();
echo $child->lexical();
echo $child->parentValue();
echo Child::forwardedKind();
"#,
        );
        let instructions = module
            .functions
            .iter()
            .flat_map(|function| &function.blocks)
            .flat_map(|block| &block.instructions)
            .map(|instruction| &instruction.kind)
            .collect::<Vec<_>>();
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, InstructionKind::DirectMethod { .. }))
        );
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, InstructionKind::VirtualMethod { .. }))
        );
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, InstructionKind::LateStaticMethod { .. }))
        );
    }

    #[test]
    fn lowers_finally_transfers_and_replacement_chains() {
        let module = compile(
            r#"<?thp
function exits(bool $returnNow): int {
    $index: int = 0;
    while ($index < 2) {
        $index = $index + 1;
        try {
            if ($returnNow) { return $index; }
            if ($index === 1) { continue; }
            break;
        } finally {
            echo "cleanup";
        }
    }
    return 0;
}
try {
    try {
        throw new Exception("pending");
    } finally {
        throw new Exception("replacement");
    }
} catch (Exception $error) {
    throw new Exception("catch replacement");
} finally {
    echo "outer cleanup";
}
"#,
        );
        let instructions = module
            .functions
            .iter()
            .flat_map(|function| &function.blocks)
            .flat_map(|block| &block.instructions)
            .map(|instruction| &instruction.kind)
            .collect::<Vec<_>>();
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, InstructionKind::ChainPrevious { .. }))
        );
        assert!(
            module
                .functions
                .iter()
                .any(|function| function.exception_handlers.len() >= 3)
        );
    }
}

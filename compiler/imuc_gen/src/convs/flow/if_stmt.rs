use crate::prelude::*;
use ast::flow::IfElse;
use convs::ExprSolver;

pub struct IfElseConv {
    pub solver: ExprSolver,
}

impl Converter for IfElseConv {
    type Input = IfElse;
}

impl Convert<Value> for IfElseConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        // TODO: Use this solver
        let Self { solver: _solver } = self;
        let ty = std::cell::OnceCell::<Ty>::default();
        let mut final_placeholders = Vec::new();

        ctx.body_mut().push_stack_record();
        let revert_stack_and_dupli = |ctx: &mut Ctx, ptr: Ptr| -> Result<()> {
            let size = ty
                .get()
                .expect("When stack is reverted, a type should be present")
                .size_or(input.span())?;
            let stack_record = ctx
                .body()
                .stack_record()
                .expect("A stack record should be present in ifelse conversion");
            ctx.body_mut().push(Cmd::Overwrite(size, stack_record, ptr));
            ctx.body_mut().revert_stack_record(size);
            Ok(())
        };

        for if_stmt in input.ifs.iter() {
            // Convert condition
            let cond = convs::ExprConv::default()
                .with_hint(Some(Ty::bool()))
                .convert(ctx, if_stmt.cond.as_ref())?
                .ok_or_else(|| {
                    ctx.push_error(
                        ConvError::new(
                            Severity::Error,
                            if_stmt
                                .cond
                                .span()
                                .expect("expected valueless expression to return None"),
                        )
                        .with_head("If condition requires a Bool condition"),
                    );
                    SendError::new_error()
                })?;
            if !cond.ty.test_eq(&Ty::bool()) {
                ctx.push_error(ConvError::new(Severity::Error, if_stmt.span()).with_text(
                    "If expression requires a bool condition",
                    format!("Expected Bool for if condition, found {}", cond.ty.name),
                ));
                assert!(ctx.body_mut().pop_stack_record(Bytes::null()));
                return Err(SendError::new_error());
            }

            // Add the guard and jump command to the end of this if block
            let inverse_ptr = ctx.body_mut().push_stack(Bytes::byte());
            ctx.body_mut().push(Cmd::Not(NumBytes::I8, cond.ptr));
            // Placeholder for a jump command
            let jump_index = ctx.body().len();
            ctx.body_mut().push(Cmd::End);

            // Convert the if body, and add a placeholder that jumps to the end of everything
            let value = convs::BodyConv::default().convert(ctx, &if_stmt.body)?;
            let final_jump_index = ctx.body().len();
            ctx.body_mut().push(Cmd::End);
            final_placeholders.push(final_jump_index);
            // Test if type is all same for each if
            let prev_has_type = ty.get().is_some();
            let prev_type = ty.get_or_init(|| value.ty.clone());
            if prev_has_type && !prev_type.test_eq(&value.ty) {
                ctx.push_error(ConvError::new(Severity::Error, if_stmt.span()).with_text(
                    "Chained ifs type mismatch",
                    format!("Required {}, found {}", prev_type.name, value.ty.name),
                ));
                assert!(ctx.body_mut().pop_stack_record(Bytes::null()));
                return Err(SendError::new_error());
            }

            // Replace the first placeholder to a proper jump command when if fails
            let current_index = ctx.body().len();
            *ctx.body_mut()
                .get_mut(jump_index)
                .expect("Body should contains a jump command placeholder") = Cmd::JumpIf(
                inverse_ptr,
                Bytes::new(
                    current_index
                        .try_into()
                        .expect("Stack length should not exceed Bytes limit"),
                ),
            );

            // Revert stack location before the next if/else branch
            revert_stack_and_dupli(ctx, value.ptr)?;
        }
        let prev_type = ty
            .get()
            .expect("The ifs is a nonempty vector, thus a type should exist when it reaches else");
        let ret_type_size = prev_type.size_or(input.span())?;

        // Convert final else stmt, if any
        if let Some(else_stmt) = &input.end {
            let value = convs::BodyConv::default().convert(ctx, else_stmt)?;
            if !prev_type.test_eq(&value.ty) {
                ctx.push_error(ConvError::new(Severity::Error, else_stmt.span()).with_text(
                    "Chained ifs type mismatch with else",
                    format!("Required {}, found {}", prev_type.name, value.ty.name),
                ));
                assert!(ctx.body_mut().pop_stack_record(Bytes::null()));
                return Err(SendError::new_error());
            }
            revert_stack_and_dupli(ctx, value.ptr)?;
            let final_ptr = Bytes::new_usize(ctx.body().len());
            for index in final_placeholders.into_iter() {
                *ctx.body_mut()
                    .get_mut(index)
                    .expect("Cmd placeholder index should exist") = Cmd::Jump(final_ptr);
            }
        } else if !prev_type.test_eq(&Ty::unit()) {
            // If there is no else branch and there is an return value, an error is thrown
            ctx.push_error(ConvError::new(Severity::Error, input.span()).with_text(
                "Expected returned value on all pathes, but there is no else branch",
                format!("Expected {}, but else branch is missing", prev_type.name),
            ));
            assert!(ctx.body_mut().pop_stack_record(Bytes::null()));
            return Err(SendError::new_error());
        }

        ctx.body_mut().pop_stack_record(ret_type_size);
        // Locate the final result ptr
        let ptr = ctx.body_mut().stack() - ret_type_size;
        Ok(Value {
            ptr,
            ty: prev_type.clone(),
        })
    }
}

#![allow(unsafe_code)] // ffi code
extern crate pq_sys;

use std::ffi::CString;
use std::os::raw as libc;
use std::ptr;

use super::result::PgResult;
use super::statement_cache::PrepareForCache;
use crate::pg::PgTypeMetadata;
use crate::result::QueryResult;
use crate::IntoSql;

use super::raw::RawConnection;

pub(crate) struct Statement {
    name: CString,
    param_formats: Vec<libc::c_int>,
}

impl Statement {
    pub(super) fn execute(
        &self,
        raw_connection: &mut RawConnection,
        param_data: &[Option<Vec<u8>>],
        row_by_row: bool,
    ) -> QueryResult<PgResult> {
        let params_pointer = param_data
            .iter()
            .map(|data| {
                data.as_ref()
                    .map(|d| d.as_ptr() as *const libc::c_char)
                    .unwrap_or(ptr::null())
            })
            .collect::<Vec<_>>();
        let param_lengths = param_data
            .iter()
            .map(|data| data.as_ref().map(|d| d.len().try_into()).unwrap_or(Ok(0)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| crate::result::Error::SerializationError(Box::new(e)))?;
        let param_count: libc::c_int = params_pointer
            .len()
            .try_into()
            .map_err(|e| crate::result::Error::SerializationError(Box::new(e)))?;
        
        unsafe {
            // Always use send_query_params for unnamed statements
            if self.name.to_bytes().is_empty() {
                raw_connection.send_query_params(
                    self.into_sql().as_ptr(),
                    param_count,
                    params_pointer.as_ptr(),
                    param_lengths.as_ptr(),
                    self.param_formats.as_ptr(),
                    1,
                )
            }
            // For named statements, use send_query_prepared
            raw_connection.send_query_prepared(
                self.name.as_ptr(),
                param_count,
                params_pointer.as_ptr(),
                param_lengths.as_ptr(),
                self.param_formats.as_ptr(),
                1,
            );
        }?;
        
        if row_by_row {
            raw_connection.enable_row_by_row_mode()?;
        }
        Ok(raw_connection.get_next_result()?.expect("Is never none"))
    }

    pub(super) fn prepare(
        raw_connection: &mut RawConnection,
        sql: &str,
        is_cached: PrepareForCache,
        param_types: &[PgTypeMetadata],
    ) -> QueryResult<Self> {
        let query_name = match is_cached {
            PrepareForCache::Yes { counter } => Some(format!("__diesel_stmt_{counter}")),
            PrepareForCache::No => None,
        };
        let name = query_name.as_deref();
        let name = CString::new(name.unwrap_or(""))?;
        let sql = CString::new(sql)?;
        let param_types_vec = param_types
            .iter()
            .map(|x| x.oid())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| crate::result::Error::SerializationError(Box::new(e)))?;

        let internal_result = unsafe {
            let param_count: libc::c_int = param_types
                .len()
                .try_into()
                .map_err(|e| crate::result::Error::SerializationError(Box::new(e)))?;
            raw_connection.prepare(
                name.as_ptr(),
                sql.as_ptr(),
                param_count,
                param_types_to_ptr(Some(&param_types_vec)),
            )
        };
        PgResult::new(internal_result?, raw_connection)?;

        Ok(Statement {
            name,
            param_formats: vec![1; param_types.len()],
        })
    }
}

fn param_types_to_ptr(param_types: Option<&Vec<u32>>) -> *const pq_sys::Oid {
    param_types
        .map(|types| types.as_ptr())
        .unwrap_or(ptr::null())
}

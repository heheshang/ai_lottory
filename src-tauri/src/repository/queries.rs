//! Query Building and Execution
//!
//! Provides flexible query building capabilities for repositories.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query operator types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    ILike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
    Contains,
    Starts,
    Ends,
}

/// Query condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCondition {
    pub field: String,
    pub operator: QueryOperator,
    pub value: Option<QueryValue>,
}

impl QueryCondition {
    pub fn new(field: &str, operator: QueryOperator) -> Self {
        Self {
            field: field.to_string(),
            operator,
            value: None,
        }
    }

    pub fn with_value(mut self, value: QueryValue) -> Self {
        self.value = Some(value);
        self
    }

    pub fn equals(field: &str, value: QueryValue) -> Self {
        Self::new(field, QueryOperator::Equals).with_value(value)
    }

    pub fn not_equals(field: &str, value: QueryValue) -> Self {
        Self::new(field, QueryOperator::NotEquals).with_value(value)
    }

    pub fn greater_than(field: &str, value: QueryValue) -> Self {
        Self::new(field, QueryOperator::GreaterThan).with_value(value)
    }

    pub fn less_than(field: &str, value: QueryValue) -> Self {
        Self::new(field, QueryOperator::LessThan).with_value(value)
    }

    pub fn like(field: &str, value: QueryValue) -> Self {
        Self::new(field, QueryOperator::Like).with_value(value)
    }

    pub fn in_list(field: &str, values: Vec<QueryValue>) -> Self {
        Self::new(field, QueryOperator::In).with_value(QueryValue::List(values))
    }

    pub fn is_null(field: &str) -> Self {
        Self::new(field, QueryOperator::IsNull)
    }

    pub fn is_not_null(field: &str) -> Self {
        Self::new(field, QueryOperator::IsNotNull)
    }
}

/// Query value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(chrono::DateTime<chrono::Utc>),
    List(Vec<QueryValue>),
    Null,
}

impl From<String> for QueryValue {
    fn from(value: String) -> Self {
        QueryValue::String(value)
    }
}

impl From<i64> for QueryValue {
    fn from(value: i64) -> Self {
        QueryValue::Integer(value)
    }
}

impl From<f64> for QueryValue {
    fn from(value: f64) -> Self {
        QueryValue::Float(value)
    }
}

impl From<bool> for QueryValue {
    fn from(value: bool) -> Self {
        QueryValue::Boolean(value)
    }
}

impl From<chrono::DateTime<chrono::Utc>> for QueryValue {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        QueryValue::DateTime(value)
    }
}

impl From<Vec<QueryValue>> for QueryValue {
    fn from(value: Vec<QueryValue>) -> Self {
        QueryValue::List(value)
    }
}

/// Logical operators for combining conditions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Query group for combining multiple conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryGroup {
    pub operator: LogicalOperator,
    pub conditions: Vec<QueryCondition>,
    pub groups: Vec<QueryGroup>,
}

impl QueryGroup {
    pub fn new(operator: LogicalOperator) -> Self {
        Self {
            operator,
            conditions: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn and() -> Self {
        Self::new(LogicalOperator::And)
    }

    pub fn or() -> Self {
        Self::new(LogicalOperator::Or)
    }

    pub fn not() -> Self {
        Self::new(LogicalOperator::Not)
    }

    pub fn with_condition(mut self, condition: QueryCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn with_group(mut self, group: QueryGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn add_condition(&mut self, condition: QueryCondition) {
        self.conditions.push(condition);
    }

    pub fn add_group(&mut self, group: QueryGroup) {
        self.groups.push(group);
    }
}

/// Sort order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Sort specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    pub field: String,
    pub order: SortOrder,
}

impl SortSpec {
    pub fn asc(field: &str) -> Self {
        Self {
            field: field.to_string(),
            order: SortOrder::Asc,
        }
    }

    pub fn desc(field: &str) -> Self {
        Self {
            field: field.to_string(),
            order: SortOrder::Desc,
        }
    }
}

/// Query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub where_clause: Option<QueryGroup>,
    pub order_by: Vec<SortSpec>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub select_fields: Option<Vec<String>>,
    pub group_by: Option<Vec<String>>,
    pub having: Option<QueryGroup>,
}

impl QueryParams {
    pub fn new() -> Self {
        Self {
            where_clause: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            select_fields: None,
            group_by: None,
            having: None,
        }
    }

    pub fn with_where(mut self, group: QueryGroup) -> Self {
        self.where_clause = Some(group);
        self
    }

    pub fn with_order(mut self, sort: SortSpec) -> Self {
        self.order_by.push(sort);
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.select_fields = Some(fields);
        self
    }

    pub fn add_order(&mut self, sort: SortSpec) {
        self.order_by.push(sort);
    }
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Query builder for fluent API
pub struct QueryBuilder {
    params: QueryParams,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            params: QueryParams::new(),
        }
    }

    pub fn where_eq(mut self, field: &str, value: QueryValue) -> Self {
        let condition = QueryCondition::equals(field, value);
        let where_clause = self.params.where_clause.unwrap_or_else(QueryGroup::and);
        let new_where_clause = where_clause.with_condition(condition);
        self.params.where_clause = Some(new_where_clause);
        self
    }

    pub fn where_ne(mut self, field: &str, value: QueryValue) -> Self {
        let condition = QueryCondition::not_equals(field, value);
        let where_clause = self.params.where_clause.unwrap_or_else(QueryGroup::and);
        let new_where_clause = where_clause.with_condition(condition);
        self.params.where_clause = Some(new_where_clause);
        self
    }

    pub fn where_gt(mut self, field: &str, value: QueryValue) -> Self {
        let condition = QueryCondition::greater_than(field, value);
        let where_clause = self.params.where_clause.unwrap_or_else(QueryGroup::and);
        let new_where_clause = where_clause.with_condition(condition);
        self.params.where_clause = Some(new_where_clause);
        self
    }

    pub fn where_lt(mut self, field: &str, value: QueryValue) -> Self {
        let condition = QueryCondition::less_than(field, value);
        let where_clause = self.params.where_clause.unwrap_or_else(QueryGroup::and);
        let new_where_clause = where_clause.with_condition(condition);
        self.params.where_clause = Some(new_where_clause);
        self
    }

    pub fn where_like(mut self, field: &str, value: QueryValue) -> Self {
        let condition = QueryCondition::like(field, value);
        let where_clause = self.params.where_clause.unwrap_or_else(QueryGroup::and);
        let new_where_clause = where_clause.with_condition(condition);
        self.params.where_clause = Some(new_where_clause);
        self
    }

    pub fn where_in(mut self, field: &str, values: Vec<QueryValue>) -> Self {
        let condition = QueryCondition::in_list(field, values);
        let where_clause = self.params.where_clause.unwrap_or_else(QueryGroup::and);
        let new_where_clause = where_clause.with_condition(condition);
        self.params.where_clause = Some(new_where_clause);
        self
    }

    pub fn where_null(mut self, field: &str) -> Self {
        let condition = QueryCondition::is_null(field);
        let where_clause = self.params.where_clause.unwrap_or_else(QueryGroup::and);
        let new_where_clause = where_clause.with_condition(condition);
        self.params.where_clause = Some(new_where_clause);
        self
    }

    pub fn where_not_null(mut self, field: &str) -> Self {
        let condition = QueryCondition::is_not_null(field);
        let where_clause = self.params.where_clause.unwrap_or_else(QueryGroup::and);
        let new_where_clause = where_clause.with_condition(condition);
        self.params.where_clause = Some(new_where_clause);
        self
    }

    pub fn order_by_asc(mut self, field: &str) -> Self {
        self.params.add_order(SortSpec::asc(field));
        self
    }

    pub fn order_by_desc(mut self, field: &str) -> Self {
        self.params.add_order(SortSpec::desc(field));
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.params.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.params.offset = Some(offset);
        self
    }

    pub fn select(mut self, fields: Vec<String>) -> Self {
        self.params.select_fields = Some(fields);
        self
    }

    pub fn build(self) -> QueryParams {
        self.params
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// SQL query builder
pub struct SqlQueryBuilder {
    table_name: String,
    params: QueryParams,
}

impl SqlQueryBuilder {
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            params: QueryParams::new(),
        }
    }

    pub fn with_params(mut self, params: QueryParams) -> Self {
        self.params = params;
        self
    }

    pub fn build_select_query(&self) -> Result<(String, Vec<QueryValue>)> {
        let mut query_parts = Vec::new();
        let mut bind_values = Vec::new();

        // SELECT clause
        let select_fields = match &self.params.select_fields {
            Some(fields) => fields.join(", "),
            None => "*".to_string(),
        };
        query_parts.push(format!("SELECT {}", select_fields));

        // FROM clause
        query_parts.push(format!("FROM {}", self.table_name));

        // WHERE clause
        if let Some(where_clause) = &self.params.where_clause {
            let (where_sql, mut values) = self.build_where_clause(where_clause)?;
            query_parts.push(format!("WHERE {}", where_sql));
            bind_values.append(&mut values);
        }

        // GROUP BY clause
        if let Some(group_by) = &self.params.group_by {
            query_parts.push(format!("GROUP BY {}", group_by.join(", ")));

            // HAVING clause
            if let Some(having) = &self.params.having {
                let (having_sql, mut values) = self.build_where_clause(having)?;
                query_parts.push(format!("HAVING {}", having_sql));
                bind_values.append(&mut values);
            }
        }

        // ORDER BY clause
        if !self.params.order_by.is_empty() {
            let order_clauses: Vec<String> = self.params
                .order_by
                .iter()
                .map(|sort| {
                    let order_str = match sort.order {
                        SortOrder::Asc => "ASC",
                        SortOrder::Desc => "DESC",
                    };
                    format!("{} {}", sort.field, order_str)
                })
                .collect();
            query_parts.push(format!("ORDER BY {}", order_clauses.join(", ")));
        }

        // LIMIT clause
        if let Some(limit) = self.params.limit {
            query_parts.push(format!("LIMIT {}", limit));
        }

        // OFFSET clause
        if let Some(offset) = self.params.offset {
            query_parts.push(format!("OFFSET {}", offset));
        }

        Ok((query_parts.join(" "), bind_values))
    }

    fn build_where_clause(&self, group: &QueryGroup) -> Result<(String, Vec<QueryValue>)> {
        let mut clause_parts = Vec::new();
        let mut bind_values = Vec::new();

        // Process conditions
        for condition in &group.conditions {
            let (condition_sql, mut values) = self.build_condition(condition)?;
            clause_parts.push(condition_sql);
            bind_values.append(&mut values);
        }

        // Process nested groups
        for nested_group in &group.groups {
            let (group_sql, mut values) = self.build_where_clause(nested_group)?;
            clause_parts.push(format!("({})", group_sql));
            bind_values.append(&mut values);
        }

        let operator_str = match group.operator {
            LogicalOperator::And => " AND ",
            LogicalOperator::Or => " OR ",
            LogicalOperator::Not => " NOT ",
        };

        let clause = if matches!(group.operator, LogicalOperator::Not) && clause_parts.len() == 1 {
            format!("NOT {}", clause_parts[0])
        } else {
            clause_parts.join(operator_str)
        };

        Ok((clause, bind_values))
    }

    fn build_condition(&self, condition: &QueryCondition) -> Result<(String, Vec<QueryValue>)> {
        match &condition.operator {
            QueryOperator::Equals => {
                let value = condition.value.as_ref()
                    .ok_or_else(|| AppError::Validation {
                        message: "Equals operator requires a value".to_string(),
                        field: "value".to_string(),
                    })?;
                Ok((format!("{} = ${}", condition.field, self.get_placeholder_index()), vec![value.clone()]))
            }
            QueryOperator::NotEquals => {
                let value = condition.value.as_ref()
                    .ok_or_else(|| AppError::Validation {
                        message: "NotEquals operator requires a value".to_string(),
                        field: "value".to_string(),
                    })?;
                Ok((format!("{} != ${}", condition.field, self.get_placeholder_index()), vec![value.clone()]))
            }
            QueryOperator::GreaterThan => {
                let value = condition.value.as_ref()
                    .ok_or_else(|| AppError::Validation {
                        message: "GreaterThan operator requires a value".to_string(),
                        field: "value".to_string(),
                    })?;
                Ok((format!("{} > ${}", condition.field, self.get_placeholder_index()), vec![value.clone()]))
            }
            QueryOperator::LessThan => {
                let value = condition.value.as_ref()
                    .ok_or_else(|| AppError::Validation {
                        message: "LessThan operator requires a value".to_string(),
                        field: "value".to_string(),
                    })?;
                Ok((format!("{} < ${}", condition.field, self.get_placeholder_index()), vec![value.clone()]))
            }
            QueryOperator::Like => {
                let value = condition.value.as_ref()
                    .ok_or_else(|| AppError::Validation {
                        message: "Like operator requires a value".to_string(),
                        field: "value".to_string(),
                    })?;
                Ok((format!("{} LIKE ${}", condition.field, self.get_placeholder_index()), vec![value.clone()]))
            }
            QueryOperator::IsNull => {
                Ok((format!("{} IS NULL", condition.field), Vec::new()))
            }
            QueryOperator::IsNotNull => {
                Ok((format!("{} IS NOT NULL", condition.field), Vec::new()))
            }
            QueryOperator::In => {
                let value = condition.value.as_ref()
                    .ok_or_else(|| AppError::Validation {
                        message: "In operator requires a value".to_string(),
                        field: "value".to_string(),
                    })?;

                if let QueryValue::List(values) = value {
                    let placeholders: Vec<String> = (0..values.len())
                        .map(|i| format!("${}", self.get_placeholder_index() + i))
                        .collect();
                    Ok((format!("{} IN ({})", condition.field, placeholders.join(", ")), values.clone()))
                } else {
                    Err(AppError::Validation {
                        message: "In operator requires a list value".to_string(),
                        field: "value".to_string(),
                    })
                }
            }
            _ => Err(AppError::NotImplemented {
                message: format!("Operator {:?} not implemented", condition.operator),
            }),
        }
    }

    fn get_placeholder_index(&self) -> usize {
        // This is simplified - in practice you'd track the actual placeholder count
        1
    }
}
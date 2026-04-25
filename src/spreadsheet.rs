use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueError {
    DivideByZero,
    CyclicalReference,
}

impl Error for ValueError {}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueError::DivideByZero => write!(f, "#DIV"),
            ValueError::CyclicalReference => write!(f, "#REF"),
        }
    }
}

// Simplified representation of a cell formula. This can be extended to include many others
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CellFormula {
    Constant(u32),
    Ref(CellIndex),
    Add([Box<CellFormula>; 2]),
    Sum(Vec<CellIndex>),
    // any others
}

pub fn ref_(x: u16, y: u16) -> CellFormula {
    let index = hash(x, y);
    CellFormula::Ref(index)
}

type CellValue = Result<u32, ValueError>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CellEntry {
    value: CellValue,
    formula: CellFormula,
}

type CellIndex = u32;

fn hash(x: u16, y: u16) -> CellIndex {
    u32::from(x) << 16 | u32::from(y) as CellIndex
}

pub struct Spreadsheet {
    // Use a hashmap to represent a sparse array of cells
    cell_arena: HashMap<CellIndex, CellEntry>,
    // Keep track of all cells referencing this cell in a formula.
    // This allows you to propagate changes without searching the whole spreadsheet.
    dependents: HashMap<CellIndex, HashSet<u32>>,
}

impl Spreadsheet {
    pub fn new() -> Self {
        return Self {
            cell_arena: HashMap::new(),
            dependents: HashMap::new(),
        };
    }

    pub fn get(self: &Self, x: u16, y: u16) -> Result<u32, ValueError> {
        let index = hash(x, y);
        self.get_at_index(&index)
    }

    pub fn set(
        self: &mut Self,
        x: u16,
        y: u16,
        formula: CellFormula,
    ) -> () {
        let index = hash(x, y); // 65537

        let dependencies = self.cell_dependencies(&formula); // { 0 }
        dbg!(&dependencies);

        if let Some(old) = self.cell_arena.insert(
            index,
            CellEntry {
                value: Ok(0),
                formula: formula,
            },
        ) {
            // clear the old cell's dependencies
            let old_dependencies = self.cell_dependencies(&old.formula);
            for dep in old_dependencies {
                if let Some(set) = self.dependents.get_mut(&dep) {
                    set.remove(&index);
                }
            }
        }

        for dep in dependencies {
            if let Some(cell_dependants) = self.dependents.get_mut(&dep) {
                cell_dependants.insert(index);
            } else {
                self.dependents.insert(dep, HashSet::from([index]));
            }
        }

        self.update_cell(&index);

        ()
    }

    fn get_at_index(self: &Self, index: &CellIndex) -> Result<u32, ValueError> {
        match self.cell_arena.get(&index) {
            Some(cell) => cell.value.clone(),
            None => Ok(0),
        }
    }

    fn update_cell(self: &mut Self, index: &CellIndex) -> () {
        let mut queue = VecDeque::<CellIndex>::from([*index]);
        let mut cycles = HashSet::<CellIndex>::new();

        while let Some(index) = queue.pop_front() {
            if cycles.contains(&index) {
                continue;
            }
            // evaluate what the cell value should be
            let value: CellValue = if self.check_circular_dependencies(&index) {
                cycles.insert(index);
                Err(ValueError::CyclicalReference)
            } else {
                if let Some(entry) = self.cell_arena.get(&index) {
                    self.evaluate_formula(&entry.formula)
                } else {
                    Ok(0)
                }
            };

            if let Some(entry) = self.cell_arena.get_mut(&index) {
                entry.value = value;
            }

            if let Some(cell_dependents) = self.dependents.get(&index) {
                for cdi in cell_dependents {
                    // TODO (optimization) if a duplicate is already in the queue, remove it and push to the back
                    if !cycles.contains(&cdi) {
                        queue.push_back(cdi.clone());
                    }
                }
            }
        }

        ()
    }

    fn evaluate_formula(self: &Self, formula: &CellFormula) -> Result<u32, ValueError> {
        println!("evaluate_formula");
        match formula {
            CellFormula::Constant(x) => Ok(*x),
            CellFormula::Ref(index) => self.get_at_index(index),
            CellFormula::Add([l, r]) => {
                let l_value = self.evaluate_formula(l)?;
                let r_value = self.evaluate_formula(r)?;
                Ok(l_value + r_value)
            }
            CellFormula::Sum(indeces) => indeces.iter().try_fold(0, |mem, &index| {
                let value = self.get_at_index(&index)?;
                Ok(mem + value)
            }),
        }
    }

    fn cell_dependencies(self: &Self, formula: &CellFormula) -> HashSet<CellIndex> {
        let deps = match formula {
            CellFormula::Constant(_) => HashSet::new(),
            CellFormula::Ref(index) => HashSet::from([*index]),
            CellFormula::Add([l, r]) => {
                let left = self.cell_dependencies(&*l);
                let right = self.cell_dependencies(&*r);
                left.union(&right).copied().collect()
            }
            CellFormula::Sum(indeces) => indeces.iter().copied().collect(),
        };
        dbg!(&deps);
        deps
    }

    /// perform a BFS to check for any self references
    fn check_circular_dependencies(self: &Self, pivot: &CellIndex) -> bool {
        // TODO: Cache intermittent values
        let mut queue: VecDeque<CellIndex> = VecDeque::from([*pivot]);
        let mut visited: HashSet<CellIndex> = HashSet::from([*pivot]);

        while queue.len() > 0 {
            dbg!(&queue);
            dbg!(&visited);
            let index = queue.pop_front().unwrap();
            let formula = match self.cell_arena.get(&index) {
                Some(entry) => {
                    match &entry.value {
                        Err(ValueError::CyclicalReference) => return true,
                        Err(_) => &entry.formula,
                        Ok(_) => &entry.formula,
                    }
                }
                None => continue,
            };
            let dependencies = self.cell_dependencies(formula);
            for dep in dependencies {
                if visited.contains(&dep) {
                    return true;
                }
                visited.insert(dep.clone());
                queue.push_back(dep);
            }
        }

        false
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set() {
        let mut sheet = Spreadsheet::new();
        assert_eq!(sheet.get(0, 0), CellValue::Ok(0));
        sheet.set(0, 0, CellFormula::Constant(1));
        assert_eq!(sheet.get(0, 0), CellValue::Ok(1));
    }

    #[test]
    fn test_ref() {
        let mut sheet = Spreadsheet::new();
        sheet.set(0, 0, CellFormula::Constant(1));
        sheet.set(0, 1, ref_(0, 0));
        assert_eq!(sheet.get(0, 1), CellValue::Ok(1));
    }

    #[test]
    fn test_ref_self() {
        let mut sheet = Spreadsheet::new();
        sheet.set(0, 0, ref_(0, 0));
        assert_eq!(sheet.get(0, 0), CellValue::Err(ValueError::CyclicalReference));
    }

    #[test]
    fn test_add() {
        let mut sheet = Spreadsheet::new();
        sheet.set(0, 0, CellFormula::Constant(1));
        sheet.set(0, 1, CellFormula::Constant(2));
        sheet.set(0, 2, CellFormula::Add([
                Box::from(ref_(0, 0)),
                Box::from(ref_(0, 1)),
        ]));
        assert_eq!(sheet.get(0, 2), CellValue::Ok(3));
    }

    #[test]
    fn test_add_ooo() {
        let mut sheet = Spreadsheet::new();
        sheet.set(0, 2, CellFormula::Add([
                Box::from(ref_(0, 0)),
                Box::from(ref_(0, 1)),
        ]));
        sheet.set(0, 0, CellFormula::Constant(1));
        sheet.set(0, 1, CellFormula::Constant(2));
        assert_eq!(sheet.get(0, 2), CellValue::Ok(3));

        sheet.set(0, 1, CellFormula::Constant(100));
        assert_eq!(sheet.get(0, 2), CellValue::Ok(101));
    }
}

"""
QuickStart Example for IndustryDB

This example demonstrates basic usage of IndustryDB with SQLite.
"""

import industrydb as idb
import polars as pl


def main():
    print("IndustryDB Quickstart Example")
    print("=" * 50)
    print(f"Version: {idb.__version__}\n")

    # Create a SQLite database connection
    print("1. Creating SQLite connection...")
    config = idb.DatabaseConfig(db_type="sqlite", path="./example.db")

    # Use context manager for automatic cleanup
    with idb.Connection(config) as conn:
        print("   ✓ Connected!\n")

        # Create a table
        print("2. Creating table...")
        conn.execute("""
            CREATE TABLE IF NOT EXISTS employees (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                department TEXT,
                salary REAL
            )
        """)
        print("   ✓ Table created!\n")

        # Insert data using DataFrame
        print("3. Inserting data...")
        df = pl.DataFrame(
            {
                "id": [1, 2, 3, 4],
                "name": ["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince"],
                "department": ["Engineering", "Sales", "Engineering", "Marketing"],
                "salary": [95000.0, 75000.0, 85000.0, 80000.0],
            }
        )

        rows = conn.insert("employees", df)
        print(f"   ✓ Inserted {rows} rows!\n")

        # Query data
        print("4. Querying all employees...")
        result = conn.execute("SELECT * FROM employees")
        print(result)
        print()

        # Query with filter
        print("5. Querying Engineering department...")
        result = conn.select("employees", where_clause="department = 'Engineering'")
        print(result)
        print()

        # Aggregation query
        print("6. Average salary by department...")
        result = conn.execute("""
            SELECT department, AVG(salary) as avg_salary
            FROM employees
            GROUP BY department
            ORDER BY avg_salary DESC
        """)
        print(result)
        print()

        # Update data
        print("7. Updating salary...")
        conn.execute("UPDATE employees SET salary = salary * 1.1 WHERE id = 1")
        result = conn.execute("SELECT * FROM employees WHERE id = 1")
        print(f"   Updated: {result}")
        print()

        # Delete data
        print("8. Deleting a record...")
        conn.execute("DELETE FROM employees WHERE id = 4")
        result = conn.execute("SELECT COUNT(*) as count FROM employees")
        print(f"   Remaining employees: {result['count'][0]}")

    print("\n✓ Connection closed automatically!")
    print("\nQuickstart completed successfully!")


if __name__ == "__main__":
    main()

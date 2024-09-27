using System.Data;
using System.Text;

namespace Unilake.ProxyQuery.TestSuite;

public static class Extensions
{
    /// <summary>
    /// Generates a textual representation of the data of <paramref name="table"/>.
    /// </summary>
    /// <param name="table">The table to print.</param>
    /// <returns>A textual representation of the data of <paramref name="table"/>.</returns>
    public static String Print(this DataTable table)
    {
        String GetCellValueAsString(DataRow row, DataColumn column)
        {
            var cellValue = row[column];
            var cellValueAsString = cellValue is null or DBNull ? "NULL" : cellValue.ToString();

            return cellValueAsString ?? String.Empty;
        }

        var columnWidths = table.Columns.Cast<DataColumn>().ToDictionary(column => column, column => column.ColumnName.Length);

        foreach (DataRow row in table.Rows)
        {
            foreach (DataColumn column in table.Columns)
            {
                columnWidths[column] = Math.Max(columnWidths[column], GetCellValueAsString(row, column).Length);
            }
        }

        var resultBuilder = new StringBuilder();

        resultBuilder.Append("| ");

        foreach (DataColumn column in table.Columns)
        {
            resultBuilder.Append(column.ColumnName.PadRight(columnWidths[column]));
            resultBuilder.Append(" | ");
        }

        resultBuilder.AppendLine();

        foreach (DataRow row in table.Rows)
        {
            resultBuilder.Append("| ");

            foreach (DataColumn column in table.Columns)
            {
                resultBuilder.Append(GetCellValueAsString(row, column).PadRight(columnWidths[column]));
                resultBuilder.Append(" | ");
            }
            resultBuilder.AppendLine();
        }

        return resultBuilder.Replace(" \n", "\n").ToString();
    }
}
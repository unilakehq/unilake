using System;
using System.Data;
using System.Data.SqlClient;

namespace Unilake.ProxyQuery.TestSuite;

public class Runner
{
    public DataTable RunQuery(string query)
    {
        string connectionString = "Server=localhost;Database=master;User Id=sa;Password=<YourStrong@Passw0rd>;";
        DataTable dataTable = new DataTable();

        using (SqlConnection connection = new SqlConnection(connectionString))
        using (SqlCommand command = new SqlCommand(query, connection))
        {
            connection.Open();
            SqlDataAdapter adapter = new SqlDataAdapter(command);
            adapter.Fill(dataTable);
            connection.Close();
        }

        return dataTable;
    }
}

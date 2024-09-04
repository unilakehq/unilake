using System;
using System.Data.SqlClient;

namespace Unilake.ProxyQuery.TestSuite;

public class Runner
{
    public Runner()
    {

    }

    public SqlDataReader RunQuery(string query)
    {
        string connectionString = "Server=localhost;Database=myDataBase;User Id=myUsername;Password=myPassword;";

        using (SqlConnection connection = new SqlConnection(connectionString))
        using (SqlCommand command = new SqlCommand(query, connection))
        {
            connection.Open();
            SqlDataReader reader = command.ExecuteReader();
            return reader;
        }
    }
}

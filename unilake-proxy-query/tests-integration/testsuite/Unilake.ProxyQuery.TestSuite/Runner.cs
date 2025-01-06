using System.Data;
using System.Data.SqlClient;

namespace Unilake.ProxyQuery.TestSuite;

public class Runner
{
    public List<string> Messages { get; private set; } = new();

    public DataTable ExecuteQueryDatatable(string query)
    {
        string connectionString = "Server=localhost;Database=master;User Id=sa;Password=<YourStrong@Passw0rd>;";
        DataTable dataTable = new DataTable();

        void OnMessage(object sender, SqlInfoMessageEventArgs e)
        {
            Messages.Add(e.Message);
        }

        using SqlConnection connection = new SqlConnection(connectionString);
        using SqlCommand command = new SqlCommand(query, connection);

        connection.InfoMessage += OnMessage;
        connection.FireInfoMessageEventOnUserErrors = true;
        connection.Open();
        SqlDataAdapter adapter = new SqlDataAdapter(command);
        adapter.Fill(dataTable);
        connection.Close();

        return dataTable;
    }
}
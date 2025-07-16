# OpenStack Cost Dashboard

A modern Rust web application that fetches OpenStack rating data and displays it in a beautiful Chart.js dashboard. Built with a clean, modular architecture for maintainability and extensibility.

## Features

- **Real-time Data Fetching**: Automatically fetches OpenStack rating data using the OpenStack CLI
- **Interactive Charts**: 
  - Bar/Pie chart showing cost by service
  - Doughnut chart showing top 5 services
  - Detailed service table with costs and percentages
- **Auto-refresh**: Data refreshes every 5 minutes automatically
- **Manual Refresh**: Click the "Refresh Data" button to update data immediately
- **Responsive Design**: Works on desktop and mobile devices
- **Configurable**: Environment-based configuration for flexibility
- **Health Monitoring**: Built-in health check and application info endpoints
- **Modular Architecture**: Clean separation of concerns with dedicated modules

## Prerequisites

- Rust (latest stable version)
- OpenStack CLI configured and authenticated
- Access to OpenStack rating service

## Installation

1. Clone this repository:
   ```bash
   git clone <repository-url>
   cd openstack-rating-graph
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   cargo run
   ```

The server will start on `http://localhost:3001`

## Usage

1. **Start the server**: Run `cargo run` in the project directory
2. **Open the dashboard**: Navigate to `http://localhost:3001` in your web browser
3. **View the data**: The dashboard will show:
   - Total monthly cost
   - Number of active services
   - Average cost per service
   - Interactive charts
   - Detailed service table

## API Endpoints

- `GET /` - Main dashboard HTML page
- `GET /api/data` - JSON data for charts
- `GET /api/refresh` - Manually trigger data refresh
- `GET /api/health` - Health check endpoint
- `GET /api/info` - Application information

## Data Structure

The application fetches data using this exact OpenStack command:
```bash
openstack rating dataframes get -b $(date +'%Y-%m-01T00:00:00+00:00') -c Resources -f json
```

Example command that would be executed:
```bash
openstack rating dataframes get -b 2025-01-01T00:00:00+00:00 -c Resources -f json
```

The application automatically generates the date string for the first day of the current month in the format `YYYY-MM-01T00:00:00+00:00`.

## Configuration

The application supports environment-based configuration through a `.env` file or environment variables.

### Using .env file (Recommended)

1. Copy the example configuration file:
   ```bash
   cp .env.example .env
   ```

2. Edit the `.env` file with your settings:
   ```bash
   # Server Configuration
   BIND_ADDRESS=0.0.0.0
   PORT=3001
   
   # Data Configuration
   REFRESH_INTERVAL_SECONDS=300
   CURRENCY_RATE=55.5
   
   # OpenStack Configuration
   OPENSTACK_COMMAND=openstack
   
   # OpenStack Authentication
   OS_AUTH_URL=https://your-openstack-endpoint:5000/v3
   OS_USERNAME=your-username
   OS_PASSWORD=your-password
   OS_PROJECT_ID=your-project-id
   OS_USER_DOMAIN_NAME=Default
   ```

3. Run the application:
   ```bash
   cargo run
   ```

### Environment Variables

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `BIND_ADDRESS` | `0.0.0.0` | Server bind address |
| `PORT` | `3001` | Server port |
| `REFRESH_INTERVAL_SECONDS` | `300` | Data refresh interval in seconds |
| `CURRENCY_RATE` | `55.5` | Rating to currency conversion rate |
| `OPENSTACK_COMMAND` | `openstack` | OpenStack CLI command name |
| `OS_AUTH_URL` | *(required)* | OpenStack authentication URL |
| `OS_USERNAME` | *(required)* | OpenStack username |
| `OS_PASSWORD` | *(required)* | OpenStack password |
| `OS_PROJECT_ID` | *(required)* | OpenStack project ID |
| `OS_USER_DOMAIN_NAME` | `Default` | OpenStack user domain name |

### Alternative: Direct Environment Variables

```bash
export PORT=8080
export REFRESH_INTERVAL_SECONDS=600
export CURRENCY_RATE=60.0
cargo run
```

**Note:** Environment variables take precedence over `.env` file values.

## Architecture

The application follows a clean, modular architecture:

```
src/
├── main.rs          # Application entry point and state management
├── config.rs        # Configuration management
├── models.rs        # Data structures and types
├── data.rs          # Data fetching and processing
├── handlers.rs      # HTTP request handlers
└── server.rs        # Server setup and background tasks
```

### Key Components:

- **Config**: Centralized configuration management with environment variable support
- **Models**: Type-safe data structures for OpenStack resources and chart data
- **DataService**: Handles OpenStack CLI integration and data processing
- **Handlers**: HTTP endpoint handlers for the REST API
- **Server**: Web server setup and background task management
- **AppState**: Shared application state with thread-safe data access

## Files Structure

```
.
├── src/
│   ├── main.rs          # Application entry point
│   ├── config.rs        # Configuration management
│   ├── models.rs        # Data structures
│   ├── data.rs          # Data fetching and processing
│   ├── handlers.rs      # HTTP request handlers
│   └── server.rs        # Server setup and background tasks
├── templates/
│   └── index.html       # Dashboard HTML template
├── .env                 # Environment configuration (create from .env.example)
├── .env.example         # Example environment configuration
├── .gitignore          # Git ignore file
├── Cargo.toml          # Rust project configuration
├── test-openstack.sh    # OpenStack setup test script
└── README.md           # This file
```

## Development

To modify the dashboard appearance, edit `templates/index.html`. The file contains:
- HTML structure
- CSS styling
- JavaScript for Chart.js integration
- Responsive design rules

## Troubleshooting

### OpenStack Authentication Issues

If you see errors like "Missing value auth-url required for auth plugin password", you need to configure OpenStack authentication:

1. **Source an OpenStack RC file**:
   ```bash
   source ~/openstack-rc.sh
   cargo run
   ```

2. **Set environment variables manually**:
   ```bash
   export OS_AUTH_URL=https://your-openstack-endpoint:5000/v3
   export OS_USERNAME=your-username
   export OS_PASSWORD=your-password
   export OS_PROJECT_ID=your-project-id
   export OS_USER_DOMAIN_NAME=Default
   cargo run
   ```

3. **Add to .env file**:
   ```bash
   # Add to your .env file
   OS_AUTH_URL=https://your-openstack-endpoint:5000/v3
   OS_USERNAME=your-username
   OS_PASSWORD=your-password
   OS_PROJECT_ID=your-project-id
   OS_USER_DOMAIN_NAME=Default
   ```

4. **Test OpenStack connection**:
   ```bash
   openstack server list
   ```

5. **Run the setup test script**:
   ```bash
   ./test-openstack.sh
   ```
   This script will check all requirements and guide you through the setup process.

### Other Common Issues

1. **OpenStack CLI not found**: Ensure the OpenStack CLI is installed and in your PATH
2. **Rating service not available**: The rating service might not be enabled on your OpenStack deployment
3. **No data displayed**: Check that the rating service is available and returning data
4. **Port conflicts**: If port 3001 is in use, change the PORT in your `.env` file

## License

This project is open source. Feel free to modify and distribute.

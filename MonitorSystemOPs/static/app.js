class SystemMonitor {
    constructor() {
        this.charts = {};
        this.history = {
            cpu: [],
            memory: [],
            disk: [],
            network: []
        };
        this.initCharts();
        this.startMonitoring();
    }

    initCharts() {
        const chartConfig = {
            type: 'line',
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100
                    }
                },
                elements: {
                    line: {
                        tension: 0.4
                    }
                },
                plugins: {
                    legend: {
                        display: false
                    }
                }
            }
        };

        this.charts.cpu = new Chart(
            document.getElementById('cpu-chart'),
            {
                ...chartConfig,
                data: {
                    labels: [],
                    datasets: [{
                        data: [],
                        borderColor: '#e74c3c',
                        backgroundColor: 'rgba(231, 76, 60, 0.1)',
                        fill: true
                    }]
                }
            }
        );

        this.charts.memory = new Chart(
            document.getElementById('memory-chart'),
            {
                ...chartConfig,
                data: {
                    labels: [],
                    datasets: [{
                        data: [],
                        borderColor: '#3498db',
                        backgroundColor: 'rgba(52, 152, 219, 0.1)',
                        fill: true
                    }]
                }
            }
        );

        this.charts.disk = new Chart(
            document.getElementById('disk-chart'),
            {
                ...chartConfig,
                data: {
                    labels: [],
                    datasets: [{
                        data: [],
                        borderColor: '#2ecc71',
                        backgroundColor: 'rgba(46, 204, 113, 0.1)',
                        fill: true
                    }]
                }
            }
        );

        this.charts.network = new Chart(
            document.getElementById('network-chart'),
            {
                type: 'line',
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {
                        y: {
                            beginAtZero: true
                        }
                    },
                    plugins: {
                        legend: {
                            display: true
                        }
                    }
                },
                data: {
                    labels: [],
                    datasets: [
                        {
                            label: 'RX',
                            data: [],
                            borderColor: '#9b59b6',
                            backgroundColor: 'rgba(155, 89, 182, 0.1)',
                            fill: true
                        },
                        {
                            label: 'TX',
                            data: [],
                            borderColor: '#e67e22',
                            backgroundColor: 'rgba(230, 126, 34, 0.1)',
                            fill: true
                        }
                    ]
                }
            }
        );
    }

    async fetchMetrics() {
        try {
            const response = await fetch('/metrics');
            if (!response.ok) throw new Error('Network error');

            const metrics = await response.json();
            this.updateUI(metrics);
            this.updateStatus(true, 'Подключено');

        } catch (error) {
            this.updateStatus(false, 'Ошибка подключения');
            console.error('Failed to fetch metrics:', error);
        }
    }

    updateUI(metrics) {
        document.getElementById('cpu-value').textContent = `${metrics.cpu_usage.toFixed(1)}%`;
        document.getElementById('memory-value').textContent = `${metrics.memory_usage_percent.toFixed(1)}%`;
        document.getElementById('disk-value').textContent = `${metrics.disk_usage_percent.toFixed(1)}%`;
        document.getElementById('network-value').textContent =
            `${this.formatBytes(metrics.network_rx)}/${this.formatBytes(metrics.network_tx)}`;
        document.getElementById('process-count').textContent = metrics.processes_count;

        this.updateChart('cpu', metrics.cpu_usage);
        this.updateChart('memory', metrics.memory_usage_percent);
        this.updateChart('disk', metrics.disk_usage_percent);
        this.updateChart('network', [metrics.network_rx, metrics.network_tx]);
        document.getElementById('last-update').textContent =
            `Последнее обновление: ${new Date().toLocaleTimeString()}`;
    }

    updateChart(type, value) {
        const chart = this.charts[type];
        const now = new Date().toLocaleTimeString();

        if (Array.isArray(value)) {
            // Для сети (массив значений)
            chart.data.labels.push(now);
            chart.data.datasets[0].data.push(value[0]);
            chart.data.datasets[1].data.push(value[1]);
        } else {
            chart.data.labels.push(now);
            chart.data.datasets[0].data.push(value);
        }

        if (chart.data.labels.length > 20) {
            chart.data.labels.shift();
            chart.data.datasets.forEach(dataset => dataset.data.shift());
        }

        chart.update();
    }

    updateStatus(connected, message) {
        const indicator = document.querySelector('.status-dot');
        const statusText = document.getElementById('status-text');

        indicator.className = 'status-dot ' + (connected ? 'connected' : 'error');
        statusText.textContent = message;
    }

    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    startMonitoring() {
        // Первый запрос сразу
        this.fetchMetrics();

        // Затем каждые 5 секунд
        setInterval(() => {
            this.fetchMetrics();
        }, 5000);
    }
}

document.addEventListener('DOMContentLoaded', () => {
    new SystemMonitor();
});
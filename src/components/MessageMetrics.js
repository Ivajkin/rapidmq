import React, { useEffect, useRef, useState } from 'react';
import { Bar } from 'react-chartjs-2';

const MessageMetrics = () => {
    const [data, setData] = useState({
        labels: ['Messages Published', 'Messages Consumed', 'Queue Count', 'Total Messages'],
        datasets: [{
            label: 'Message Metrics',
            data: [0, 0, 0, 0],
            backgroundColor: [
                'rgba(255, 99, 132, 0.2)',
                'rgba(54, 162, 235, 0.2)',
                'rgba(255, 206, 86, 0.2)',
                'rgba(75, 192, 192, 0.2)'
            ],
            borderColor: [
                'rgba(255, 99, 132, 1)',
                'rgba(54, 162, 235, 1)',
                'rgba(255, 206, 86, 1)',
                'rgba(75, 192, 192, 1)'
            ],
            borderWidth: 1
        }]
    });

    useEffect(() => {
        const fetchData = async () => {
            const response = await fetch('/metrics');
            const text = await response.text();
            const metrics = text.split('\n').filter(line => !line.startsWith('#'));
            const values = metrics.map(line => parseFloat(line.split(' ')[1]));
            setData(prevData => ({
                ...prevData,
                datasets: [{
                    ...prevData.datasets[0],
                    data: values
                }]
            }));
        };

        fetchData();
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    return <Bar data={data} />;
};

export default MessageMetrics;
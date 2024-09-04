function processData(data) {
    // Optimized code
    const result = new Map();
    for (const item of data) {
        if (!result.has(item.key)) {
            result.set(item.key, []);
        }
        result.get(item.key).push(item.value);
    }
    return Array.from(result.values());
}
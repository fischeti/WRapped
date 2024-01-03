function get_dimensions(id) {
  var element = document.getElementById(id);
  var x = element.offsetWidth;
  var y = element.offsetHeight;

  return { width: x, height: y };
}

function drawProgessCircle(container_id, ratio) {
  // Clear existing content
  d3.select("#" + container_id).select('svg').remove();

  // Get new dimensions of the container
  const container_dimensions = get_dimensions(container_id);
  const width = container_dimensions.width;
  const height = container_dimensions.height;
  const margin = 20;

  // The radius of the pie chart is half the smallest side
  const radius = Math.min(width, height) / 2 - margin;
  innerRadius = radius - width / 10;

  // Append the svg object to the div called 'progress-circle'
  const svg = d3.select("#" + container_id)
    .append("svg")
      .attr("width", width)
      .attr("height", height)
    .append("g")
      .attr("transform", `translate(${width / 2}, ${height / 2})`);

  const startAngle = Math.PI;
  const endAngle = ratio * 2 * Math.PI + Math.PI

  // Foreground circle (progress)
  const arc = d3.arc()
    .innerRadius(innerRadius) // Adjust for donut thickness
    .outerRadius(radius)
    .startAngle(startAngle) // Starting angle
    .endAngle(endAngle) // Ending angle
    .cornerRadius(20);

  const defs = svg.append("defs");

  const gradient = defs.append("linearGradient")
    .attr("id", "svgGradient")
    .attr("x1", "0%")
    .attr("x2", "100%")
    .attr("y1", "0%")
    .attr("y2", "0%");

  gradient.append("stop")
    .attr("offset", "0%")
    .attr("stop-color", "var(--color-palette-3)");

  gradient.append("stop")
    .attr("offset", "100%")
    .attr("stop-color", "var(--color-palette-4)");

  // Apply the gradient to the arc
  const path = svg.append("path")
    .attr("d", arc)
    .attr("fill", "url(#svgGradient)");

  path.transition()
    .duration(1500)
    .attrTween("d", function() {
      const interpolate = d3.interpolate(startAngle, endAngle);
      return function(t) {
        arc.endAngle(interpolate(t));
        return arc();
      };
    });
}

function drawBarChart(container_id, weekdayData) {
  // Clear existing content
  d3.select("#" + container_id).select('svg').remove();

  const margin = { top: 50, right: 30, bottom: 40, left: 40 };

  // Get new dimensions of the container
  const container_dimensions = get_dimensions(container_id);
  const width = container_dimensions.width - margin.left - margin.right;
  const height = container_dimensions.height - margin.top - margin.bottom;

  // Transform the data into an array format and sort by weekday
  const data = Object.keys(weekdayData).map(day => {
    return { day: (parseInt(day) + 3) % 7, count: weekdayData[day] };
  }).sort((a, b) => a.day - b.day);

  // Create SVG element
  const svg = d3.select("#" + container_id)
    .append("svg")
      .attr("width", width + margin.left + margin.right)
      .attr("height", height + margin.top + margin.bottom)
    .append("g")
      .attr("transform", `translate(${margin.left}, ${margin.top})`);

  // Define the scales
  const x = d3.scaleBand()
    .range([0, width])
    .domain(data.map(d => d.day))
    .padding(0.5);

  const y = d3.scaleLinear()
    .range([height, 0])
    .domain([0, d3.max(data, d => d.count)]);

  const xAxis = d3.axisBottom(x)
    .tickFormat(d => ["Fri", "Sat", "Sun", "Mon", "Tue", "Wed", "Thu"][d])
    .tickSize(0)
    .tickPadding(10);

  // Assuming you want ticks at specific intervals (0, 10, 20, etc.)
  const tickValues = d3.range(0, d3.max(data, d => d.count) + 5, 5);

  const yAxis = d3.axisLeft(y)
    .tickValues(tickValues)
    .tickSize(0)
    .tickPadding(10);

  // Add the X-axis
  svg.append("g")
    .attr("transform", `translate(0, ${height})`)
    .call(xAxis)
    .selectAll("text")
    .attr("fill", "var(--text-grey)")
    .attr("font-size", "14px")
    .attr("font-weight", "bold");

  // Add the Y-axis
  svg.append("g")
    .call(yAxis)
    .selectAll("text")
    .attr("fill", "var(--text-grey)")
    .attr("font-size", "16px")
    .attr("font-weight", "bold");

  // Remove the axis lines and ticks
  svg.selectAll(".domain, tick line").remove();

  // Append horizontal lines for each Y-axis tick
  svg.selectAll(".horizontal-line")
    .data(tickValues.slice(1)) // Adjust to match the number of ticks on your Y-axis
    .join("line")
      .attr("class", "horizontal-line")
      .attr("x1", 0)
      .attr("x2", width)
      .attr("y1", d => y(d))
      .attr("y2", d => y(d))
      .style("stroke", "var(--color-palette-2)")
      .style("stroke-opacity", "0.5")
      .style("stroke-width", "1px")
      .style("stroke-dasharray", ("10, 10")); // Dashed line pattern

  const defs = svg.append("defs");

  const gradient = defs.append("linearGradient")
    .attr("id", "bar-gradient")
    .attr("gradientUnits", "userSpaceOnUse")
    .attr("x1", "0%").attr("y1", "0%")
    .attr("x2", "0%").attr("y2", "100%");

  gradient.append("stop")
    .attr("offset", "50%")
    .attr("stop-color", "var(--color-palette-3)");

  gradient.append("stop")
    .attr("offset", "100%")
    .attr("stop-color", "var(--color-palette-4)");

  // Create the bars
  svg.selectAll(".bar")
  .data(data) // Adjust to match the number of ticks on your Y-axis
    .join("rect")
      .attr("class", "bar")
      .attr("x", d => x(d.day))
      .attr("y", d => height)
      .attr("width", x.bandwidth())
      .attr("height", 0)
      .attr("fill", "url(#bar-gradient)")
      .style("filter", "url(#glow)")
      .attr("rx", x.bandwidth()/2)
      .attr("ry", x.bandwidth()/2)
    .transition()
      .duration(1500)
      .attr("y", d => y(d.count))
      .attr("height", d => height - y(d.count));
}

function drawTimeOfDayChart(container_id, timeOfDayData) {
  // Clear existing content
  d3.select("#" + container_id).select('svg').remove();

  const margin = { top: 50, right: 30, bottom: 40, left: 40 };

  // Get new dimensions of the container
  const container_dimensions = get_dimensions(container_id);
  const width = container_dimensions.width - margin.left - margin.right;
  const height = container_dimensions.height - margin.top - margin.bottom;


  // Transform into an array of objects
  const data = Object.entries(timeOfDayData).map(([hour, count]) => {
    return { hour: parseInt(hour), count };
  });

  // Sort data by hour
  data.sort((a, b) => a.hour - b.hour);

  // Create SVG element
  const svg = d3.select("#" + container_id)
    .append("svg")
      .attr("width", width + margin.left + margin.right)
      .attr("height", height + margin.top + margin.bottom)
    .append("g")
      .attr("transform", `translate(${margin.left}, ${margin.top})`);

  // Define the scales
  const x = d3.scaleLinear()
    .domain([0, 24]) // 24 hours in a day
    .range([0, width]);

  const y = d3.scaleLinear()
    .domain([0, d3.max(data, d => d.count)]) // Max count of reports
    .range([height, 0]);

  const hourTicks = d3.range(0, 24, 3);
  const formatHourTick = d => {
    return d === 24 ? `0:00` : `${d}:00`;
  };

  const xAxis = d3.axisBottom(x)
    .tickValues(hourTicks)
    .tickFormat(formatHourTick)
    .tickSize(0)
    .tickPadding(15);

  const yAxis = d3.axisLeft(y)
    .ticks(5)
    .tickSize(0)
    .tickPadding(20);

  // Add the X-axis
  svg.append("g")
    .attr("transform", `translate(0, ${height})`)
    .call(xAxis)
    .selectAll("text")
    .attr("fill", "var(--text-grey)")
    .attr("font-size", "14px");
    // .attr("font-weight", "bold");


  // Add the Y-axis
  svg.append("g")
    .call(yAxis)
    .selectAll("text")
    .attr("fill", "var(--text-grey)")
    .attr("font-size", "16px")
    .attr("font-weight", "bold");


  // Remove the axis lines and ticks
  svg.selectAll(".domain, tick line").remove();

  // Append horizontal lines for each Y-axis tick
  svg.selectAll(".vertical-line")
    .data(hourTicks) // Adjust to match the number of ticks on your Y-axis
    .join("line")
      .attr("class", "vertical-line")
      .attr("x1", d => x(d))
      .attr("x2", d => x(d))
      .attr("y1", 0)
      .attr("y2", height)
      .style("stroke", "var(--color-palette-2)")
      .style("stroke-opacity", "0.5")
      .style("stroke-width", "1px")
      .style("stroke-dasharray", ("10, 10")); // Dashed line pattern

    const defs = svg.append("defs");

    const areaGradient = defs.append("linearGradient")
      .attr("id", "area-gradient")
      .attr("gradientUnits", "userSpaceOnUse")
      .attr("x1", "0%").attr("y1", "0%")
      .attr("x2", "0%").attr("y2", "100%");

    areaGradient.append("stop")
      .attr("offset", "50%")
      .attr("stop-color", "var(--color-palette-3)");

      areaGradient.append("stop")
      .attr("offset", "100%")
      .attr("stop-color", "var(--color-palette-4)");

    // Define a clip path
    const clip = defs.append("clipPath")
      .attr("id", "clip")
      .append("rect")
        .attr("width", 0)
        .attr("height", height + margin.top + margin.bottom);

    const area = d3.area()
      .x(d => x(d.hour))
      .y0(height)
      .y1(d => y(d.count))
      .curve(d3.curveCardinal);

    svg.append("path")
      .datum(data)
      .attr("class", "area")
      .attr("d", area)
      // .attr("height", height + margin.top + margin.bottom)
      .attr("clip-path", "url(#clip)")
      .attr("fill", "url(#area-gradient)");

    const line = d3.line()
      .x(d => x(d.hour))
      .y(d => y(d.count))
      .curve(d3.curveCardinal); // Creates a smooth line

    svg.append("path")
      .datum(data)
      .attr("fill", "none")
      .attr("clip-path", "url(#clip)")
      .attr("stroke","var(--text-grey)")
      .attr("stroke-width", 2.5)
      .attr("d", line);

    // Animate the clip path
    svg.select("#clip rect")
      .transition()
      .duration(2000)
      .attr("width", width + margin.left + margin.right);
}

function winnerEmojis(i) {
  if (i == 1) return "🏆";
  if (i == 2) return "🥈";
  if (i == 3) return "🥉";
  return ".";
}

function updateCCList(ccData) {
  // Transform the data into an array and sort it
  const sortedData = Object.entries(ccData)
    .map(([username, count]) => ({ username, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10); // Only show the top 10


  // Create list elements
  const listContainer = document.getElementById('cc-list');
  sortedData.forEach((item, index) => {
    const listItem = document.createElement('li');
    listItem.style.animationDelay = `${index * 0.15}s`;

    const placeSpan = document.createElement('div');
    placeSpan.className = 'cc-list-place';
    placeSpan.textContent = `${winnerEmojis(index + 1)}`;

    const usernameSpan = document.createElement('div');
    usernameSpan.className = 'cc-list-username';
    usernameSpan.textContent = `@${item.username}`;

    const countSpan = document.createElement('div');
    countSpan.className = 'cc-list-count';
    countSpan.textContent = `${item.count}x`;

    listItem.appendChild(placeSpan);
    listItem.appendChild(usernameSpan);
    listItem.appendChild(countSpan);

    listContainer.appendChild(listItem);
  });
}

document.addEventListener('DOMContentLoaded', function() {
  const numWrsWrittenId = 'num-wrs-written';
  const numWrsSkippedId = 'num-wrs-skipped';
  const progressCircleId = 'progress-circle';
  const ratioTextOverlayId = 'ratio-text-overlay';
  const delayOfReplyId = 'delay-of-reply';
  const weekdayId = 'weekday-chart-container';
  const timeofdayId = 'timeofday-chart-container';
  let ratioRepliedWRs;
  let weekdayData;
  let timeofdayData;
  let numWrsWritten;
  let numWrsSkipped;
  let ccData;

  const numWrsWrittenContainer= document.getElementById(numWrsWrittenId);
  function updateNumWrsWritten(numWrs) {
    const numWrsText = numWrs + " WRs"
    numWrsWrittenContainer.textContent = numWrsText;
  }

  const numWrsSkippedContainer = document.getElementById(numWrsSkippedId);
  function updateNumWrsSkipped(numWrs) {
    const numWrsText = numWrs + " WRs"
    numWrsSkippedContainer.textContent = numWrsText;
  }

  function resizeProgressCircleChart() {
    if (ratioRepliedWRs !== undefined) {
      drawProgessCircle(progressCircleId, ratioRepliedWRs);
    }
  }
  const ratioTextOverlayContainer = document.getElementById(ratioTextOverlayId);
  function updateTextOverlay(ratio) {
    const ratioText = (ratio * 100).toFixed() + '%';
    ratioTextOverlayContainer.textContent = ratioText;
  }

  function resizeWeekdayChart() {
    if (weekdayData !== undefined) {
      drawBarChart(weekdayId, weekdayData);
    }
  }

  function resizeTimeOfDayChart() {
    if (timeofdayData !== undefined) {
      drawTimeOfDayChart(timeofdayId, timeofdayData);
    }
  }

  const delayOfReplyContainer = document.getElementById(delayOfReplyId);
  function updateDelay(delayDays) {
    const delayDaysText = delayDays.toFixed(1) + " days"
    delayOfReplyContainer.textContent = delayDaysText;
  }

  fetch('../shared/stats.json')
    .then(response => response.json())
    .then(data => {
        // Now we have the JSON data
        ratioRepliedWRs = data.ratio_replied_wrs;
        numWrsWritten = data.num_wrs;
        numWrsSkipped = data.num_skipped_wrs;
        delayDays = data.avg_reply_delay;
        weekdayData = data.weekday_wr_histogram;
        timeofdayData = data.hour_reply_histogram;
        ccData = data.cc_histogram;
        updateNumWrsWritten(numWrsWritten);
        updateNumWrsSkipped(numWrsSkipped);
        updateTextOverlay(ratioRepliedWRs);
        updateDelay(delayDays);
        updateCCList(ccData);
        resizeProgressCircleChart(progressCircleId, ratioRepliedWRs);
        resizeWeekdayChart(weekdayId, weekdayData);
        resizeTimeOfDayChart(timeofdayId, timeofdayData);
        // Set up the resize event listener now that we have data
        window.addEventListener('resize', resizeProgressCircleChart);
        window.addEventListener('resize', resizeWeekdayChart);
        window.addEventListener('resize', resizeTimeOfDayChart);
    })
    .catch(error => {
        console.error('Error fetching data:', error);
        ratioTextOverlayContainer.textContent = 'Sth failed.';
        numWrsWrittenContainer.textContent = 'Sth failed.';
        numWrsSkippedContainer.textContent = 'Sth failed.';
        delayOfReplyContainer.textContent = 'Sth failed.';
    });
});

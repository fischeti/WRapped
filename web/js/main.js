function drawProgessCircle(container, ratio) {
  // Clear existing content
  d3.select('#progress-circle').select('svg').remove();

  // Get new dimensions of the container
  const width = container.clientWidth;
  const height = container.clientHeight;
  const margin = 20;

  // The radius of the pie chart is half the smallest side
  const radius = Math.min(width, height) / 2 - margin;
  innerRadius = radius - 25;

  // Append the svg object to the div called 'progress-circle'
  const svg = d3.select("#progress-circle")
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


  // Create a drop shadow filter
  const dropShadowFilter = defs.append("filter")
    .attr("id", "drop-shadow")
    .attr("height", "130%"); // To accommodate the shadow

  dropShadowFilter.append("feGaussianBlur")
    .attr("in", "SourceAlpha")
    .attr("stdDeviation", 0.5) // Adjust for blur size
    .attr("result", "blur");

  dropShadowFilter.append("feOffset")
    .attr("in", "blur")
    .attr("dx", 1) // Horizontal offset
    .attr("dy", 1) // Vertical offset
    .attr("result", "offsetBlur");

  const feMerge = dropShadowFilter.append("feMerge");
    feMerge.append("feMergeNode")
    .attr("in", "offsetBlur");
    feMerge.append("feMergeNode")
    .attr("in", "SourceGraphic");

  // Apply the gradient to the arc
  const path = svg.append("path")
    .attr("d", arc)
    .attr("fill", "url(#svgGradient)")
    .style("filter", "url(#drop-shadow)");

  path.transition()
    .duration(2000)
    .attrTween("d", function() {
      const interpolate = d3.interpolate(startAngle, endAngle);
      return function(t) {
        arc.endAngle(interpolate(t));
        return arc();
      };
    });
}

document.addEventListener('DOMContentLoaded', function() {
  const container = document.getElementById('progress-circle');
  let ratioRepliedWRs; // Variable to store the data

  function resizeChart() {
    if (ratioRepliedWRs !== undefined) {
      drawProgessCircle(container, ratioRepliedWRs);
    }
  }

  d3.json('../../shared/stats.json').then(function(data) {
    ratioRepliedWRs = data.ratio_replied_wrs;
    drawProgessCircle(container, ratioRepliedWRs);

    // Set up the resize event listener now that we have data
    window.addEventListener('resize', resizeChart);
  }).catch(function(error) {
    console.error('Error loading the JSON file:', error);
  });
});

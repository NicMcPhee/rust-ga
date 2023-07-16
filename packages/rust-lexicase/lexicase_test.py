import time
from pyshgp.gp.individual import Individual
from pyshgp.gp.population import Population
from pyshgp.gp.selection import Lexicase
import os
import random
import numpy as np
import rust_lexicase

os.environ["OPENBLAS_NUM_THREADS"] = "1"

print("Measuring python lexicase")

# t0 = time.perf_counter()
# t1 = time.perf_counter()

POP_SIZE = 1000

population = []

for _ in range(POP_SIZE):
    individual = Individual(None, None)
    individual.error_vector = np.array(random.choices(range(1000), k=100))
    # print("Making parent")
    # print(individual.error_vector)
    population.append(individual)

# individual = Individual(None, None)
# individual.error_vector = np.array([0, 0, 0, 0, 0, 10, 10, 10, 10, 10])
# population.append(individual)

print(population)

pop = Population(population)
lexicase = Lexicase()

NUM_ITER = 1000

evs = []

t0 = time.perf_counter()
# for _ in range(NUM_ITER):
#     # print("Selecing child")
#     child = lexicase.select_one(pop)
#     # evs.append(child.error_vector.tolist())
#     # print(child.error_vector)

children = lexicase.select(pop, NUM_ITER)

t1 = time.perf_counter()

# evs.sort()

# for ev in evs:
#     print(ev)

took = (t1 - t0) / NUM_ITER

print(f"Python took an avg of {took * 1000:.5f}ms per iteration")

evs = []

t0 = time.perf_counter()
# for _ in range(NUM_ITER):
#     # print("Selecing child")
#     child = rust_lexicase.select_one(pop)
#     # evs.append(child.error_vector.tolist())
#     # print(child.error_vector)

children = rust_lexicase.select(pop, NUM_ITER)

t1 = time.perf_counter()

# evs.sort()

# for ev in evs:
#     print(ev)

took = (t1 - t0) / NUM_ITER

print(f"  Rust took an avg of {took * 1000:.5f}ms per iteration")
